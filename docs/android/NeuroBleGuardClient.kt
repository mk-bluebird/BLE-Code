package org.aln.ble.guard

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

@Serializable
sealed class FfiIntent {
    @Serializable
    @SerialName("scan")
    data object Scan : FfiIntent()

    @Serializable
    @SerialName("connect")
    data class Connect(
        val class_id: String,
        val device_id: String,
    ) : FfiIntent()

    @Serializable
    @SerialName("subscribe_characteristic")
    data class SubscribeCharacteristic(
        val class_id: String,
        val device_id: String,
        val service_uuid: String,
        val characteristic_uuid: String,
    ) : FfiIntent()

    @Serializable
    @SerialName("write_characteristic")
    data class WriteCharacteristic(
        val class_id: String,
        val device_id: String,
        val service_uuid: String,
        val characteristic_uuid: String,
        val payload_len: Int,
    ) : FfiIntent()
}

@Serializable
data class FfiLinkParams(
    val phy: String,
    val encrypted: Boolean,
    val mic_present: Boolean,
    val bonded: Boolean,
    val conn_interval_ms: Int,
    val max_pdu_bytes: Int,
    val cte_present: Boolean,
)

@Serializable
data class BleGuardRequest(
    val intent: FfiIntent,
    val link: FfiLinkParams? = null,
)

@Serializable
data class BleGuardResponse(
    val decision: String,
    val reason: String? = null,
)

class NeuroBleGuardClient(
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    private val client = OkHttpClient()
    private val mediaTypeJson = "application/json; charset=utf-8".toMediaType()

    suspend fun evaluate(request: BleGuardRequest): BleGuardResponse {
        val payload = json.encodeToString(BleGuardRequest.serializer(), request)
        val body = payload.toRequestBody(mediaTypeJson)

        val httpRequest = Request.Builder()
            .url("http://127.0.0.1:8765/ble-guard") // local bridge endpoint
            .post(body)
            .build()

        client.newCall(httpRequest).execute().use { response ->
            val text = response.body?.string().orEmpty()
            return json.decodeFromString(BleGuardResponse.serializer(), text)
        }
    }
}
