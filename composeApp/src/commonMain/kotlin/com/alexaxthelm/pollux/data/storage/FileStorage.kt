package com.alexaxthelm.pollux.data.storage

interface FileStorage {
    /** Returns the full file path to use for storing a downloaded episode. */
    fun getDownloadPath(episodeId: String): String

    fun fileExists(path: String): Boolean

    fun deleteFile(path: String)

    fun writeBytes(path: String, bytes: ByteArray)
}

/**
 * Converts an arbitrary episode ID (often a URL like "https://example.com/ep/1")
 * into a string that is safe to use as a filename component on all supported platforms.
 * Any character that isn't alphanumeric, a dot, a hyphen, or an underscore is replaced
 * with an underscore.
 */
internal fun episodeIdToFilename(episodeId: String): String =
    episodeId.replace(Regex("[^A-Za-z0-9._-]"), "_")

/** Returns the platform-specific FileStorage implementation. */
expect fun createPlatformFileStorage(): FileStorage
