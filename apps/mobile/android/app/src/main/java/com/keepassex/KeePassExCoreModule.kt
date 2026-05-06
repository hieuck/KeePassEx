/**
 * KeePassEx Android — React Native Native Module
 * Bridges React Native to the Rust core library via JNI
 */
package com.keepassex

import com.facebook.react.bridge.*
import com.facebook.react.module.annotations.ReactModule
import kotlinx.coroutines.*

@ReactModule(name = KeePassExCoreModule.NAME)
class KeePassExCoreModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    companion object {
        const val NAME = "KeePassExCore"

        // Load Rust JNI library
        init {
            System.loadLibrary("keepassex_core")
        }
    }

    override fun getName() = NAME

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // ─── JNI Declarations ────────────────────────────────────────────────────

    private external fun nativeOpenVault(path: String, password: String, keyFileData: ByteArray?): String
    private external fun nativeCreateVault(path: String, name: String, password: String): String
    private external fun nativeCloseVault()
    private external fun nativeLockVault()
    private external fun nativeGetEntries(groupUuid: String?): String
    private external fun nativeGetEntry(uuid: String, includePassword: Boolean): String
    private external fun nativeGetEntryPassword(uuid: String): String
    private external fun nativeCreateEntry(argsJson: String): String
    private external fun nativeUpdateEntry(argsJson: String): String
    private external fun nativeDeleteEntry(uuid: String, permanent: Boolean): String
    private external fun nativeSearchEntries(query: String): String
    private external fun nativeGenerateTotp(entryUuid: String): String
    private external fun nativeGeneratePassword(argsJson: String): String
    private external fun nativeAuditVault(): String
    private external fun nativeGetGroups(): String

    // ─── Vault Operations ─────────────────────────────────────────────────────

    @ReactMethod
    fun openVault(path: String, password: String, keyFileData: ReadableArray?, promise: Promise) {
        scope.launch {
            try {
                val keyBytes = keyFileData?.let { arr ->
                    ByteArray(arr.size()) { i -> arr.getInt(i).toByte() }
                }
                val result = nativeOpenVault(path, password, keyBytes)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    val map = Arguments.createMap().apply {
                        putString("name", json.getString("name"))
                        putString("description", json.optString("description"))
                        putInt("entryCount", json.getInt("entry_count"))
                        putInt("groupCount", json.getInt("group_count"))
                        putString("path", path)
                    }
                    promise.resolve(map)
                } else {
                    promise.reject("VAULT_ERROR", json.optString("error", "Failed to open vault"))
                }
            } catch (e: Exception) {
                promise.reject("VAULT_ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun createVault(path: String, name: String, password: String, promise: Promise) {
        scope.launch {
            try {
                val result = nativeCreateVault(path, name, password)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    val map = Arguments.createMap().apply {
                        putString("name", name)
                        putString("description", "")
                        putInt("entryCount", 0)
                        putInt("groupCount", 1)
                        putString("path", path)
                    }
                    promise.resolve(map)
                } else {
                    promise.reject("VAULT_ERROR", json.optString("error"))
                }
            } catch (e: Exception) {
                promise.reject("VAULT_ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun closeVault(promise: Promise) {
        nativeCloseVault()
        promise.resolve(null)
    }

    @ReactMethod
    fun lockVault() {
        nativeLockVault()
    }

    // ─── Entry Operations ─────────────────────────────────────────────────────

    @ReactMethod
    fun getEntries(groupUuid: String?, promise: Promise) {
        scope.launch {
            try {
                val result = nativeGetEntries(groupUuid)
                val json = org.json.JSONArray(result)
                promise.resolve(jsonArrayToWritableArray(json))
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun getEntry(uuid: String, includePassword: Boolean, promise: Promise) {
        scope.launch {
            try {
                val result = nativeGetEntry(uuid, includePassword)
                val json = org.json.JSONObject(result)
                promise.resolve(jsonObjectToWritableMap(json))
            } catch (e: Exception) {
                promise.reject("NOT_FOUND", e.message)
            }
        }
    }

    @ReactMethod
    fun getEntryPassword(uuid: String, promise: Promise) {
        scope.launch {
            try {
                val result = nativeGetEntryPassword(uuid)
                promise.resolve(result)
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun createEntry(args: ReadableMap, promise: Promise) {
        scope.launch {
            try {
                val argsJson = readableMapToJson(args).toString()
                val result = nativeCreateEntry(argsJson)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    promise.resolve(json.getString("uuid"))
                } else {
                    promise.reject("ERROR", json.optString("error"))
                }
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun updateEntry(args: ReadableMap, promise: Promise) {
        scope.launch {
            try {
                val argsJson = readableMapToJson(args).toString()
                val result = nativeUpdateEntry(argsJson)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    promise.resolve(null)
                } else {
                    promise.reject("ERROR", json.optString("error"))
                }
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun deleteEntry(uuid: String, permanent: Boolean, promise: Promise) {
        scope.launch {
            try {
                val result = nativeDeleteEntry(uuid, permanent)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    promise.resolve(null)
                } else {
                    promise.reject("ERROR", json.optString("error"))
                }
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    @ReactMethod
    fun searchEntries(query: String, promise: Promise) {
        scope.launch {
            try {
                val result = nativeSearchEntries(query)
                val json = org.json.JSONArray(result)
                promise.resolve(jsonArrayToWritableArray(json))
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    // ─── OTP ─────────────────────────────────────────────────────────────────

    @ReactMethod
    fun generateTotp(entryUuid: String, promise: Promise) {
        try {
            val result = nativeGenerateTotp(entryUuid)
            val json = org.json.JSONObject(result)
            if (json.getBoolean("success")) {
                val map = Arguments.createMap().apply {
                    putString("code", json.getString("code"))
                    putInt("remainingSeconds", json.getInt("remaining_seconds"))
                    putInt("period", json.getInt("period"))
                    putDouble("progress", json.getDouble("progress"))
                }
                promise.resolve(map)
            } else {
                promise.reject("OTP_ERROR", json.optString("error"))
            }
        } catch (e: Exception) {
            promise.reject("OTP_ERROR", e.message)
        }
    }

    // ─── Generator ───────────────────────────────────────────────────────────

    @ReactMethod
    fun generatePassword(args: ReadableMap, promise: Promise) {
        scope.launch {
            try {
                val argsJson = readableMapToJson(args).toString()
                val result = nativeGeneratePassword(argsJson)
                val json = org.json.JSONObject(result)
                if (json.getBoolean("success")) {
                    val map = Arguments.createMap().apply {
                        putString("password", json.getString("password"))
                        putDouble("entropy", json.getDouble("entropy"))
                        putInt("strengthScore", json.getInt("strength_score"))
                        putString("strengthLabel", json.getString("strength_label"))
                    }
                    promise.resolve(map)
                } else {
                    promise.reject("ERROR", json.optString("error"))
                }
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    // ─── Health ───────────────────────────────────────────────────────────────

    @ReactMethod
    fun auditVault(promise: Promise) {
        scope.launch {
            try {
                val result = nativeAuditVault()
                val json = org.json.JSONObject(result)
                promise.resolve(jsonObjectToWritableMap(json))
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    // ─── Groups ───────────────────────────────────────────────────────────────

    @ReactMethod
    fun getGroups(promise: Promise) {
        scope.launch {
            try {
                val result = nativeGetGroups()
                val json = org.json.JSONArray(result)
                promise.resolve(jsonArrayToWritableArray(json))
            } catch (e: Exception) {
                promise.reject("ERROR", e.message)
            }
        }
    }

    // ─── Helpers ─────────────────────────────────────────────────────────────

    private fun jsonObjectToWritableMap(json: org.json.JSONObject): WritableMap {
        val map = Arguments.createMap()
        val keys = json.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            when (val value = json.get(key)) {
                is String -> map.putString(key, value)
                is Int -> map.putInt(key, value)
                is Long -> map.putInt(key, value.toInt())
                is Double -> map.putDouble(key, value)
                is Boolean -> map.putBoolean(key, value)
                is org.json.JSONObject -> map.putMap(key, jsonObjectToWritableMap(value))
                is org.json.JSONArray -> map.putArray(key, jsonArrayToWritableArray(value))
                else -> map.putString(key, value.toString())
            }
        }
        return map
    }

    private fun jsonArrayToWritableArray(json: org.json.JSONArray): WritableArray {
        val array = Arguments.createArray()
        for (i in 0 until json.length()) {
            when (val value = json.get(i)) {
                is String -> array.pushString(value)
                is Int -> array.pushInt(value)
                is Double -> array.pushDouble(value)
                is Boolean -> array.pushBoolean(value)
                is org.json.JSONObject -> array.pushMap(jsonObjectToWritableMap(value))
                is org.json.JSONArray -> array.pushArray(jsonArrayToWritableArray(value))
                else -> array.pushString(value.toString())
            }
        }
        return array
    }

    private fun readableMapToJson(map: ReadableMap): org.json.JSONObject {
        val json = org.json.JSONObject()
        val iterator = map.keySetIterator()
        while (iterator.hasNextKey()) {
            val key = iterator.nextKey()
            when (map.getType(key)) {
                ReadableType.String -> json.put(key, map.getString(key))
                ReadableType.Number -> json.put(key, map.getDouble(key))
                ReadableType.Boolean -> json.put(key, map.getBoolean(key))
                ReadableType.Map -> json.put(key, readableMapToJson(map.getMap(key)!!))
                ReadableType.Array -> json.put(key, readableArrayToJson(map.getArray(key)!!))
                ReadableType.Null -> json.put(key, org.json.JSONObject.NULL)
            }
        }
        return json
    }

    private fun readableArrayToJson(array: ReadableArray): org.json.JSONArray {
        val json = org.json.JSONArray()
        for (i in 0 until array.size()) {
            when (array.getType(i)) {
                ReadableType.String -> json.put(array.getString(i))
                ReadableType.Number -> json.put(array.getDouble(i))
                ReadableType.Boolean -> json.put(array.getBoolean(i))
                ReadableType.Map -> json.put(readableMapToJson(array.getMap(i)))
                ReadableType.Array -> json.put(readableArrayToJson(array.getArray(i)))
                ReadableType.Null -> json.put(org.json.JSONObject.NULL)
            }
        }
        return json
    }
}
