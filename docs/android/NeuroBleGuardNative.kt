package org.aln.ble.guard

object NeuroBleGuardNative {
    init {
        System.loadLibrary("ble_android_ffi")
    }

    external fun initBleGuardFromProfileJson(profileJson: String): Boolean

    external fun evaluateBleGuardJson(requestJson: String): String
}

suspend fun guardedConnectExample(
    classId: String,
    deviceId: String,
    link: FfiLinkParams,
): BleGuardResponse {
    val request = BleGuardRequest(
        intent = FfiIntent.Connect(classId = classId, deviceId = deviceId),
        link = link,
    )

    val json = Json.encodeToString(BleGuardRequest.serializer(), request)
    val respJson = NeuroBleGuardNative.evaluateBleGuardJson(json)
    return Json.decodeFromString(BleGuardResponse.serializer(), respJson)
}
