package io.github.paxel.iibrain

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import java.io.File

/**
 * Files on their way into the Brain. The core takes a Capture's files by
 * path (docs/adr/0011), so what another app hands over as a content URI
 * is copied into the app's cache first; the core copies it on into the
 * Brain's folder, and the phone app then drops the copy here.
 */
object Captures {
    /** Where files wait until the core has taken them. */
    fun dir(context: Context): File =
        File(context.cacheDir, "captures").apply { mkdirs() }

    /** A new, empty file for a photo the camera app is about to write. */
    fun newPhoto(context: Context): File =
        File(dir(context), "photo-${System.currentTimeMillis()}.jpg")

    /** Copies what [uri] names into the cache and says where it lies. */
    fun copyIn(context: Context, uri: Uri): String? = try {
        val target = free(dir(context), nameOf(context, uri))
        context.contentResolver.openInputStream(uri)?.use { input ->
            target.outputStream().use { output -> input.copyTo(output) }
        }
        target.absolutePath.takeIf { target.length() > 0 }
    } catch (_: Exception) {
        null
    }

    /**
     * What another app shared: its text, and its files copied into the
     * cache. Null when the intent shares nothing.
     */
    fun shared(context: Context, intent: Intent?): Map<String, Any?>? {
        if (intent == null) return null
        val uris: List<Uri> = when (intent.action) {
            Intent.ACTION_SEND -> listOfNotNull(stream(intent))
            Intent.ACTION_SEND_MULTIPLE -> streams(intent)
            else -> return null
        }
        val text = intent.getStringExtra(Intent.EXTRA_TEXT).orEmpty()
        val files = uris.mapNotNull { copyIn(context, it) }
        if (text.isBlank() && files.isEmpty()) return null
        return mapOf("text" to text, "files" to files)
    }

    @Suppress("DEPRECATION")
    private fun stream(intent: Intent): Uri? =
        intent.getParcelableExtra(Intent.EXTRA_STREAM) as? Uri

    @Suppress("DEPRECATION")
    private fun streams(intent: Intent): List<Uri> =
        intent.getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM).orEmpty()

    /** The file name the other app gives for [uri], made safe. */
    private fun nameOf(context: Context, uri: Uri): String {
        val shown = try {
            context.contentResolver
                .query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
                ?.use { cursor ->
                    if (cursor.moveToFirst()) cursor.getString(0) else null
                }
        } catch (_: Exception) {
            null
        }
        val name = (shown ?: uri.lastPathSegment ?: "file")
            .substringAfterLast('/')
            .replace(Regex("[\\\\:*?\"<>|\\p{Cntrl}]"), " ")
            .trim()
        return name.ifEmpty { "file" }
    }

    /** A file named [name] in [dir], or with a number when that is taken. */
    fun free(dir: File, name: String): File {
        var file = File(dir, name)
        val stem = name.substringBeforeLast('.', name)
        val ending = if ('.' in name) "." + name.substringAfterLast('.') else ""
        var n = 2
        while (file.exists()) {
            file = File(dir, "$stem $n$ending")
            n++
        }
        return file
    }
}
