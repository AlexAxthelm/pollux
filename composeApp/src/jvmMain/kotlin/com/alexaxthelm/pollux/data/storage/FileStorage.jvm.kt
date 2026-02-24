package com.alexaxthelm.pollux.data.storage

import java.io.File

actual fun createPlatformFileStorage(): FileStorage = JvmFileStorage()

class JvmFileStorage(baseDir: File = File(System.getProperty("user.home"), ".pollux")) : FileStorage {
    private val downloadsDir = File(baseDir, "downloads").also { it.mkdirs() }

    override fun getDownloadPath(episodeId: String): String =
        File(downloadsDir, "$episodeId.mp3").absolutePath

    override fun fileExists(path: String): Boolean = File(path).exists()

    override fun deleteFile(path: String) {
        File(path).delete()
    }

    override fun writeBytes(path: String, bytes: ByteArray) {
        File(path).writeBytes(bytes)
    }
}
