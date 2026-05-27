#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEScan.h>
#include <BLEAdvertisedDevice.h>
#include <string>
#include <vector>

// Governance structures (mirroring ALN schemas)
struct NeuralProfile {
    BLEUUID serviceUUID;
    int rohCeiling;
    int baseRoH;
    std::vector<std::string> allowedDeviceIds;
};

// Observation struct matching ble-model
struct BleDeviceObservation {
    std::string deviceId;
    std::string name;
    int16_t rssiDbm;
    std::vector<std::string> serviceUuids;
};

class BLEGovernanceScanner : public BLEAdvertisedDeviceCallbacks {
public:
    BLEGovernanceScanner(NeuralProfile profile) : profile(profile) {}

    void onResult(BLEAdvertisedDevice advertisedDevice) override {
        // Only accept devices that provide our target service
        if (!advertisedDevice.haveServiceUUID() ||
            !advertisedDevice.isAdvertisingService(profile.serviceUUID)) {
            return;
        }
        // Check allowed device list (if not empty)
        std::string addr = advertisedDevice.getAddress().toString();
        if (!profile.allowedDeviceIds.empty()) {
            bool found = false;
            for (const auto& id : profile.allowedDeviceIds) {
                if (addr == id) { found = true; break; }
            }
            if (!found) return;
        }
        // Collect observation (no RoH check on scan, only on connect)
        BleDeviceObservation obs;
        obs.deviceId = addr;
        obs.name = advertisedDevice.haveName() ? advertisedDevice.getName() : "";
        obs.rssiDbm = advertisedDevice.getRSSI();
        obs.serviceUuids.push_back(profile.serviceUUID.toString());
        // Store or process observation
        observations.push_back(obs);
    }

    std::vector<BleDeviceObservation> getObservations() const { return observations; }

private:
    NeuralProfile profile;
    std::vector<BleDeviceObservation> observations;
};

// Main governed connection: check RoH before connecting
bool governedConnect(BLEClient*& pClient, BLEAdvertisedDevice device, NeuralProfile& profile) {
    int currentRoH = profile.baseRoH;
    if (currentRoH > profile.rohCeiling) {
        Serial.println("Connection denied: RoH ceiling exceeded");
        return false;
    }
    pClient = BLEDevice::createClient();
    if (!pClient->connect(&device)) {
        Serial.println("Connection failed");
        return false;
    }
    // Discover and subscribe to neural characteristic (NUS RX)
    BLERemoteService* pRemoteService = pClient->getService(profile.serviceUUID);
    if (pRemoteService == nullptr) {
        pClient->disconnect();
        return false;
    }
    // Assuming the RX characteristic UUID is known (e.g., 6E400001-...)
    BLERemoteCharacteristic* pRxChar = pRemoteService->getCharacteristic(
        BLEUUID("6E400001-B5A3-F393-E0A9-E50E24DCCA9E"));
    if (pRxChar == nullptr) {
        pClient->disconnect();
        return false;
    }
    if(pRxChar->canNotify()) {
        pRxChar->registerForNotify([](BLERemoteCharacteristic* pChar, uint8_t* pData, size_t length, bool isNotify) {
            // Process neural data (e.g., log or stream)
            Serial.write(pData, length);
        });
    }
    return true;
}

void setup() {
    Serial.begin(115200);
    BLEDevice::init("");
    NeuralProfile openbciProfile{
        BLEUUID("6E400001-B5A3-F393-E0A9-E50E24DCCA9E"), // NUS service
        5,  // rohCeiling
        2,  // baseRoH (e.g., raw signal + streaming)
        {}   // empty allowedDeviceIds -> any device allowed
    };

    BLEScan* pBLEScan = BLEDevice::getScan();
    BLEGovernanceScanner scanner(openbciProfile);
    pBLEScan->setAdvertisedDeviceCallbacks(&scanner);
    pBLEScan->setActiveScan(true);
    pBLEScan->start(10); // scan for 10 seconds

    auto obs = scanner.getObservations();
    if (!obs.empty()) {
        // Connect to the first seen device
        // (In a real app, you'd prompt user or use a UI selection)
        // ... governedConnect call, etc.
    }
}

void loop() {
    delay(1000);
}
