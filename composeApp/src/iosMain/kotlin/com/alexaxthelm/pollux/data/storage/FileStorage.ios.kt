package com.alexaxthelm.pollux.data.storage

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import platform.Foundation.NSData
import platform.Foundation.NSDocumentDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSUserDomainMask
import platform.Foundation.create
import platform.Foundation.writeToFile

actual fun createPlatformFileStorage(): FileStorage = IosFileStorage()

@OptIn(ExperimentalForeignApi::class)
class IosFileStorage : FileStorage {
    private val downloadsDir: String

    init {
        val docDir = NSFileManager.defaultManager.URLForDirectory(
            directory = NSDocumentDirectory,
            inDomain = NSUserDomainMask,
            appropriateForURL = null,
            create = true,
            error = null,
        )!!.path!!
        downloadsDir = "$docDir/downloads"
        NSFileManager.defaultManager.createDirectoryAtPath(
            path = downloadsDir,
            withIntermediateDirectories = true,
            attributes = null,
            error = null,
        )
    }

    override fun getDownloadPath(episodeId: String): String =
        "$downloadsDir/${episodeIdToFilename(episodeId)}.mp3"

    override fun fileExists(path: String): Boolean =
        NSFileManager.defaultManager.fileExistsAtPath(path)

    override fun deleteFile(path: String) {
        NSFileManager.defaultManager.removeItemAtPath(path, error = null)
    }

    override fun writeBytes(path: String, bytes: ByteArray) {
        val nsData = bytes.usePinned { pinned ->
            NSData.create(bytes = pinned.addressOf(0), length = bytes.size.toULong())
        }
        nsData.writeToFile(path, atomically = true)
    }
}
