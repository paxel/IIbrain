package io.github.paxel.iibrain

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.Settings
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * The phone app's one screen. Everything the app does is the core's
 * (docs/adr/0008); here is only what Android alone can answer.
 */
class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "iibrain/storage")
            .setMethodCallHandler { call, result ->
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
    }

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
                1,
            )
        }
    }
}
