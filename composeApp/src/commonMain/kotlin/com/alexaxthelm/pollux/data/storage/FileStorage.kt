package com.alexaxthelm.pollux.data.storage

interface FileStorage {
    /** Returns the full file path to use for storing a downloaded episode. */
    fun getDownloadPath(episodeId: String): String

    fun fileExists(path: String): Boolean

    fun deleteFile(path: String)

    fun writeBytes(path: String, bytes: ByteArray)
}

/** Returns the platform-specific FileStorage implementation. */
expect fun createPlatformFileStorage(): FileStorage
