package s

public class BleGovernanceService extends Service {
    private BluetoothLeScanner scanner;
    private NeuralProfile profile;
    private final IBinder binder = new LocalBinder();
    private BluetoothGatt gatt;

    public class LocalBinder extends Binder {
        BleGovernanceService getService() {
            return BleGovernanceService.this;
        }
    }

    @Override
    public IBinder onBind(Intent intent) {
        return binder;
    }

    @Override
    public void onCreate() {
        super.onCreate();
        BluetoothManager manager = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
        BluetoothAdapter adapter = manager.getAdapter();
        scanner = adapter.getBluetoothLeScanner();
        // Load profile from resources or intent (here hardcoded for OpenBCI NUS)
        profile = new NeuralProfile(
                UUID.fromString("6E400001-B5A3-F393-E0A9-E50E24DCCA9E"),
                5, 2, null
        );
    }

    // Governed scan: only devices with target service and in allowed list
    public void startGovernedScan(ScanCallback callback) {
        if (scanner == null) return;
        ScanFilter filter = new ScanFilter.Builder()
                .setServiceUuid(new ParcelUuid(profile.serviceUuid))
                .build();
        ScanSettings settings = new ScanSettings.Builder()
                .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
                .build();
        scanner.startScan(Collections.singletonList(filter), settings, new ScanCallback() {
            @Override
            public void onScanResult(int callbackType, ScanResult result) {
                if (profile.allowedDeviceIds != null && !profile.allowedDeviceIds.isEmpty()) {
                    if (!profile.allowedDeviceIds.contains(result.getDevice().getAddress())) {
                        return;
                    }
                }
                callback.onScanResult(callbackType, result);
            }
        });
    }

    // Governed connect
    public void governedConnect(BluetoothDevice device, int additionalRoh, BluetoothGattCallback gattCallback) {
        int totalRoh = profile.baseRoH + additionalRoh;
        if (totalRoh > profile.rohCeiling) {
            throw new SecurityException("RoH ceiling exceeded");
        }
        gatt = device.connectGatt(this, false, gattCallback);
    }

    // Subscribe to neural data (NUS RX characteristic)
    public void subscribeToData(BluetoothGatt gatt, UUID serviceUuid, UUID charUuid) {
        BluetoothGattService service = gatt.getService(serviceUuid);
        if (service == null) return;
        BluetoothGattCharacteristic characteristic = service.getCharacteristic(charUuid);
        if (characteristic == null) return;
        gatt.setCharacteristicNotification(characteristic, true);
        BluetoothGattDescriptor descriptor = characteristic.getDescriptor(
                UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"));
        if (descriptor != null) {
            descriptor.setValue(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE);
            gatt.writeDescriptor(descriptor);
        }
    }

    @Override
    public void onDestroy() {
        if (gatt != null) {
            gatt.close();
            gatt = null;
        }
        super.onDestroy();
    }
}

class NeuralProfile {
    UUID serviceUuid;
    int rohCeiling;
    int baseRoH;
    Set<String> allowedDeviceIds;

    NeuralProfile(UUID serviceUuid, int rohCeiling, int baseRoH, Set<String> allowedDeviceIds) {
        this.serviceUuid = serviceUuid;
        this.rohCeiling = rohCeiling;
        this.baseRoH = baseRoH;
        this.allowedDeviceIds = allowedDeviceIds;
    }
}
