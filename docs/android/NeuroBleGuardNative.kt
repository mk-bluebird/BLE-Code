package org.aln.ble.guard

object NeuroBleGuardNative {
    init {
        System.loadLibrary("ble_android_ffi")
    }

    external fun initBleGuardFromProfileJson(profileJson: String): Boolean

    external fun evaluateBleGuardJson(requestJson: String): String
}
