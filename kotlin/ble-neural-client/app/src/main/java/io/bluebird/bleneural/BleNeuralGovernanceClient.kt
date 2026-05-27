package io.bluebird.bleneural

import android.bluetooth.*
import android.content.Context
import android.os.ParcelUuid
import java.util.UUID

/**
 * Governance-enforcing BLE client for neural devices (e.g., OpenBCI Cyton).
 * Analogous to crates/ble-guard in Rust.
 */
class BleNeuralGovernanceClient(
    private val context: Context,
    private val profile: NeuralProfile,
    private val bluetoothAdapter: BluetoothAdapter
) {
    private var bluetoothGatt: BluetoothGatt? = null
    private val handler = android.os.Handler(android.os.Looper.getMainLooper())

    // Governance check before scanning
    fun startGovernedScan(callback: (BleDeviceObservation) -> Unit) {
        if (profile.rohCeiling < 0) {
            throw GovernanceInvariantError("ROH ceiling invalid")
        }
        val scanner = bluetoothAdapter.bluetoothLeScanner ?: return
        val filter = ScanFilter.Builder()
            .setServiceUuid(ParcelUuid(profile.serviceUuid)) // e.g., NUS service
            .build()
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()

        scanner.startScan(listOf(filter), settings, object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                val record = result.scanRecord ?: return
                val device = result.device
                val obs = BleDeviceObservation(
                    deviceId = device.address,
                    name = device.name,
                    rssiDbm = result.rssi,
                    serviceUuids = record.serviceUuids?.map { it.uuid } ?: emptyList()
                )
                if (profile.allowedDeviceIds?.contains(device.address) != false) {
                    callback(obs)
                }
            }
        })
    }

    // Connect with RoH accumulation check
    fun governedConnect(device: BluetoothDevice, onConnected: (BluetoothGatt) -> Unit) {
        val currentRoH = profile.baseRoH // simplified; would sum from service policies
        if (currentRoH > profile.rohCeiling) {
            throw RoHLimitExceeded("Request would exceed RoH ceiling")
        }
        bluetoothGatt = device.connectGatt(context, false, object : BluetoothGattCallback() {
            override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
                if (newState == BluetoothProfile.STATE_CONNECTED) {
                    onConnected(gatt)
                }
            }
        }, BluetoothDevice.TRANSPORT_LE)
    }

    fun subscribeToNeuralStream(gatt: BluetoothGatt, serviceUuid: UUID, charUuid: UUID) {
        val service = gatt.getService(serviceUuid) ?: throw IllegalStateException("Service not found")
        val characteristic = service.getCharacteristic(charUuid)
        gatt.setCharacteristicNotification(characteristic, true)
        // Enable indication/notification descriptor if needed
        characteristic.getDescriptor(CLIENT_CHARACTERISTIC_CONFIG_UUID)?.let {
            gatt.writeDescriptor(it, BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
        }
    }

    companion object {
        val CLIENT_CHARACTERISTIC_CONFIG_UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
    }
}

data class NeuralProfile(
    val serviceUuid: UUID,
    val rohCeiling: Int,
    val baseRoH: Int = 0,
    val allowedDeviceIds: Set<String>? = null,
    val allowedCharacteristics: Set<UUID> = emptySet()
)

data class BleDeviceObservation(
    val deviceId: String,
    val name: String?,
    val rssiDbm: Int,
    val serviceUuids: List<UUID>
)

class GovernanceInvariantError(message: String) : Exception(message)
class RoHLimitExceeded(message: String) : Exception(message)
