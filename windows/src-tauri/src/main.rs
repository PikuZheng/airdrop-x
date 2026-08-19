#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    collections::HashMap,
    io::Write,
    net::{TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use windows::{
    Devices::{
        Bluetooth::Advertisement::{
            BluetoothLEAdvertisementPublisher, BluetoothLEAdvertisementReceivedEventArgs,
            BluetoothLEAdvertisementWatcher, BluetoothLEManufacturerData, BluetoothLEScanningMode,
        },
        Enumeration::DeviceInformation,
        WiFiDirect::{WiFiDirectDevice, WiFiDirectDeviceSelectorType},
    },
    Foundation::TypedEventHandler,
    Storage::Streams::DataWriter,
};

const COMPANY_ID: u16 = 0xfffe;

#[derive(Serialize, Clone)]
struct NearbyDevice {
    name: String,
    address: String,
    rssi: i16,
}

struct RadioState {
    _publisher: Option<BluetoothLEAdvertisementPublisher>,
    wifi: Mutex<Option<WiFiDirectDevice>>,
}

fn start_ble_advertising() -> windows::core::Result<BluetoothLEAdvertisementPublisher> {
    let publisher = BluetoothLEAdvertisementPublisher::new()?;
    let writer = DataWriter::new()?;
    writer.WriteBytes(b"ADX1")?;
    let data = BluetoothLEManufacturerData::new()?;
    data.SetCompanyId(COMPANY_ID)?;
    data.SetData(&writer.DetachBuffer()?)?;
    publisher.Advertisement()?.ManufacturerData()?.Append(&data)?;
    publisher.Start()?;
    Ok(publisher)
}

#[tauri::command]
async fn scan_ble() -> Result<Vec<NearbyDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let devices = Arc::new(Mutex::new(HashMap::new()));
        let output = devices.clone();
        let watcher = BluetoothLEAdvertisementWatcher::new().map_err(|error| error.to_string())?;
        watcher
            .SetScanningMode(BluetoothLEScanningMode::Active)
            .map_err(|error| error.to_string())?;
        let token = watcher
            .Received(&TypedEventHandler::<
                BluetoothLEAdvertisementWatcher,
                BluetoothLEAdvertisementReceivedEventArgs,
            >::new(move |_, args| {
                let args = args.ok()?;
                let advertisement = args.Advertisement()?;
                if advertisement
                    .GetManufacturerDataByCompanyId(COMPANY_ID)?
                    .Size()?
                    == 0
                {
                    return Ok(());
                }
                let address = args.BluetoothAddress()?;
                let name = advertisement.LocalName()?.to_string();
                output.lock().unwrap().insert(
                    address,
                    NearbyDevice {
                        name: if name.is_empty() {
                            "AirDrop-X Android".into()
                        } else {
                            name
                        },
                        address: format!("{address:012X}"),
                        rssi: args.RawSignalStrengthInDBm()?,
                    },
                );
                Ok(())
            }))
            .map_err(|error| error.to_string())?;
        watcher.Start().map_err(|error| error.to_string())?;
        std::thread::sleep(Duration::from_secs(4));
        watcher.Stop().map_err(|error| error.to_string())?;
        watcher
            .RemoveReceived(token)
            .map_err(|error| error.to_string())?;
        let result = devices.lock().unwrap().values().cloned().collect();
        Ok(result)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn connect_wifi_direct(state: tauri::State<'_, RadioState>) -> Result<String, String> {
    let device = tauri::async_runtime::spawn_blocking(|| {
        let selector =
            WiFiDirectDevice::GetDeviceSelector2(WiFiDirectDeviceSelectorType::AssociationEndpoint)
                .map_err(|error| error.to_string())?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)
            .and_then(|operation| operation.get())
            .map_err(|error| error.to_string())?;
        if devices.Size().map_err(|error| error.to_string())? == 0 {
            return Err("未发现已开启直连的 Android 设备".into());
        }
        let id = devices
            .GetAt(0)
            .and_then(|item| item.Id())
            .map_err(|error| error.to_string())?;
        WiFiDirectDevice::FromIdAsync(&id)
            .and_then(|operation| operation.get())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    let pairs = device
        .GetConnectionEndpointPairs()
        .map_err(|error| error.to_string())?;
    let address = pairs
        .GetAt(0)
        .and_then(|pair| pair.RemoteHostName())
        .and_then(|host| host.CanonicalName())
        .map_err(|error| error.to_string())?
        .to_string();
    let endpoint = format!("{address}:48765");
    for attempt in 0..10 {
        match TcpStream::connect(&endpoint).and_then(|mut stream| stream.write_all(b"ADXP")) {
            Ok(()) => break,
            Err(error) if attempt == 9 => {
                return Err(format!("直连成功，但 Android 接收端未就绪：{error}"));
            }
            Err(_) => std::thread::sleep(Duration::from_millis(300)),
        }
    }
    *state.wifi.lock().unwrap() = Some(device);
    Ok(endpoint)
}

#[tauri::command]
async fn send_file(address: String, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let stream = TcpStream::connect(address).map_err(|error| error.to_string())?;
        airdrop_x_protocol::send(stream, Path::new(&path)).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn receive_file(bind: String, directory: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let listener = TcpListener::bind(bind).map_err(|error| error.to_string())?;
        let (stream, _) = listener.accept().map_err(|error| error.to_string())?;
        airdrop_x_protocol::receive(stream, Path::new(&directory))
            .map(|path| path.display().to_string())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn main() {
    let publisher = start_ble_advertising().ok();
    tauri::Builder::default()
        .manage(RadioState {
            _publisher: publisher,
            wifi: Mutex::new(None),
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_ble,
            connect_wifi_direct,
            send_file,
            receive_file
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AirDrop-X");
}
