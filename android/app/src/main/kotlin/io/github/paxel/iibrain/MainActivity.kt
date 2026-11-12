package io.github.paxel.iibrain

import android.Manifest
import android.app.Activity
import android.content.ClipData
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.MediaStore
import android.provider.Settings
import androidx.core.content.FileProvider
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.io.File

/**
 * The phone app's one screen. Everything the app does is the core's
 * (docs/adr/0008); here is only what Android alone can do: the access to
 * the storage, the camera, the files and what other apps share, the
 * prints that wait, and the Live Lookups.
 */
class MainActivity : FlutterActivity() {
    private var capture: MethodChannel? = null

    /** The answer a camera app, a file picker or a permission is awaited for. */
    private var awaited: MethodChannel.Result? = null
    private var photo: File? = null

    /** What runs once the permission asked for is granted. */
    private var granted: (() -> Any?)? = null

    /** What another app shared and the app has not taken yet. */
    private var shared: Map<String, Any?>? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        shared = Captures.shared(this, intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        Captures.shared(this, intent)?.let {
            shared = it
            capture?.invokeMethod("shared", null)
        }
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        val messenger = flutterEngine.dartExecutor.binaryMessenger
        MethodChannel(messenger, "iibrain/storage").setMethodCallHandler { call, result ->
            when (call.method) {
                "granted" -> result.success(storageGranted())
                "request" -> {
                    requestStorage()
                    result.success(null)
                }
                // Where the Device keeps its own files: the index and
                // its settings, which never enter the Brain's folder.
                "deviceDir" -> result.success(filesDir.absolutePath)
                "root" -> result.success(
                    Environment.getExternalStorageDirectory().absolutePath,
                )
                else -> result.notImplemented()
            }
        }
        capture = MethodChannel(messenger, "iibrain/capture").also {
            it.setMethodCallHandler(::onCapture)
        }
    }

    private fun onCapture(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "takePhoto" -> takePhoto(result)
            "pickFiles" -> pickFiles(result)
            "takeShared" -> {
                result.success(shared)
                shared = null
            }
            "share" -> {
                share(
                    call.argument<String>("text").orEmpty(),
                    call.argument<List<String>>("files").orEmpty(),
                )
                result.success(null)
            }
            "waitingPrints" -> result.success(PrintInbox.waiting(this))
            "openPrintSettings" -> {
                startActivity(Intent(Settings.ACTION_PRINT_SETTINGS))
                result.success(null)
            }
            "calendar" -> withPermission(Manifest.permission.READ_CALENDAR, result) {
                Lookups.calendar(this)
            }
            "contacts" -> withPermission(Manifest.permission.READ_CONTACTS, result) {
                Lookups.contacts(this, call.argument<List<String>>("words").orEmpty())
            }
            else -> result.notImplemented()
        }
    }

    // ------------------------------------------------------- the storage

    /** Whether the app may read and write shared storage by file paths. */
    private fun storageGranted(): Boolean =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Environment.isExternalStorageManager()
        } else {
            checkSelfPermission(Manifest.permission.WRITE_EXTERNAL_STORAGE) ==
                PackageManager.PERMISSION_GRANTED
        }

    /**
     * Asks for it: the system's page for all-files access from Android 11
     * on, the permission dialog before. The app asks again when it is back.
     */
    private fun requestStorage() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            val own = Intent(
                Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                Uri.parse("package:$packageName"),
            )
            try {
                startActivity(own)
            } catch (_: Exception) {
                // Some systems have only the list of all apps.
                startActivity(Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION))
            }
        } else {
            requestPermissions(
                arrayOf(
                    Manifest.permission.READ_EXTERNAL_STORAGE,
                    Manifest.permission.WRITE_EXTERNAL_STORAGE,
                ),
                REQUEST_STORAGE,
            )
        }
    }

    // ------------------------------------------------------- capturing

    /** Has a camera app take a photo; answers with its path, or null. */
    private fun takePhoto(result: MethodChannel.Result) {
        if (awaited != null) return result.success(null)
        val file = Captures.newPhoto(this)
        val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
            .putExtra(MediaStore.EXTRA_OUTPUT, uriOf(file))
            .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        try {
            photo = file
            awaited = result
            startActivityForResult(intent, REQUEST_PHOTO)
        } catch (_: Exception) {
            // No camera app.
            awaited = null
            result.success(null)
        }
    }

    /** Has the person pick files; answers with their paths in the cache. */
    private fun pickFiles(result: MethodChannel.Result) {
        if (awaited != null) return result.success(emptyList<String>())
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT)
            .addCategory(Intent.CATEGORY_OPENABLE)
            .setType("*/*")
            .putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true)
        try {
            awaited = result
            startActivityForResult(intent, REQUEST_FILES)
        } catch (_: Exception) {
            awaited = null
            result.success(emptyList<String>())
        }
    }

    @Deprecated("The activity result API needs a ComponentActivity, which a FlutterActivity is not.")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        val result = awaited ?: return
        when (requestCode) {
            REQUEST_PHOTO -> {
                awaited = null
                val file = photo
                val taken = resultCode == Activity.RESULT_OK && file != null && file.length() > 0
                result.success(if (taken) file?.absolutePath else null)
            }
            REQUEST_FILES -> {
                awaited = null
                val uris = mutableListOf<Uri>()
                if (resultCode == Activity.RESULT_OK) {
                    data?.clipData?.let { clip ->
                        for (i in 0 until clip.itemCount) uris += clip.getItemAt(i).uri
                    }
                    if (uris.isEmpty()) data?.data?.let { uris += it }
                }
                result.success(uris.mapNotNull { Captures.copyIn(this, it) })
            }
        }
    }

    /** Hands a Note to another app: its text, and its Attachments. */
    private fun share(text: String, files: List<String>) {
        val uris = ArrayList(files.map { uriOf(File(it)) })
        val intent = when (uris.size) {
            0 -> Intent(Intent.ACTION_SEND).setType("text/plain")
            1 -> Intent(Intent.ACTION_SEND).setType("*/*")
                .putExtra(Intent.EXTRA_STREAM, uris[0])
            else -> Intent(Intent.ACTION_SEND_MULTIPLE).setType("*/*")
                .putParcelableArrayListExtra(Intent.EXTRA_STREAM, uris)
        }
        intent.putExtra(Intent.EXTRA_TEXT, text)
        if (uris.isNotEmpty()) {
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            intent.clipData = ClipData.newRawUri(null, uris[0]).apply {
                for (uri in uris.drop(1)) addItem(ClipData.Item(uri))
            }
        }
        startActivity(Intent.createChooser(intent, null))
    }

    private fun uriOf(file: File): Uri =
        FileProvider.getUriForFile(this, "$packageName.files", file)

    // ------------------------------------------------------- lookups

    /**
     * Answers with what [read] gives once [permission] is granted, asking
     * for it when it is not; with null when the person refuses.
     */
    private fun withPermission(
        permission: String,
        result: MethodChannel.Result,
        read: () -> Any?,
    ) {
        if (checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED) {
            return result.success(attempt(read))
        }
        if (awaited != null) return result.success(null)
        awaited = result
        granted = read
        requestPermissions(arrayOf(permission), REQUEST_LOOKUP)
    }

    private fun attempt(read: () -> Any?): Any? = try {
        read()
    } catch (_: Exception) {
        null
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode != REQUEST_LOOKUP) return
        val result = awaited ?: return
        val read = granted
        awaited = null
        granted = null
        val allowed = grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED
        result.success(if (allowed && read != null) attempt(read) else null)
    }

    private companion object {
        const val REQUEST_STORAGE = 1
        const val REQUEST_PHOTO = 2
        const val REQUEST_FILES = 3
        const val REQUEST_LOOKUP = 4
    }
}
