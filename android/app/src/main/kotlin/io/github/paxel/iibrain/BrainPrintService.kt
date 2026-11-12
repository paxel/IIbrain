package io.github.paxel.iibrain

import android.content.Context
import android.print.PrintAttributes
import android.print.PrinterCapabilitiesInfo
import android.print.PrinterId
import android.print.PrinterInfo
import android.printservice.PrintJob
import android.printservice.PrintService
import android.printservice.PrinterDiscoverySession
import java.io.File
import java.io.FileInputStream

/**
 * Print Capture (docs/adr/0007): IIbrain as a printer of the system. What
 * any app prints to it arrives here as a PDF, with no printer involved.
 *
 * The service runs without the app's screens, and the core is reached
 * from those (docs/adr/0011). So a printed PDF waits in the app's own
 * files, and becomes an Unfiled Note with the PDF as its Attachment when
 * the app is next opened or brought to the front.
 */
class BrainPrintService : PrintService() {
    override fun onCreatePrinterDiscoverySession(): PrinterDiscoverySession =
        object : PrinterDiscoverySession() {
            override fun onStartPrinterDiscovery(priorityList: MutableList<PrinterId>) {
                addPrinters(listOf(printer()))
            }

            override fun onStopPrinterDiscovery() = Unit

            override fun onValidatePrinters(printerIds: MutableList<PrinterId>) = Unit

            override fun onStartPrinterStateTracking(printerId: PrinterId) {
                addPrinters(listOf(printer()))
            }

            override fun onStopPrinterStateTracking(printerId: PrinterId) = Unit

            override fun onDestroy() = Unit
        }

    /** The one printer: always there, taking any page as it comes. */
    private fun printer(): PrinterInfo {
        val id = generatePrinterId("iibrain")
        val capabilities = PrinterCapabilitiesInfo.Builder(id)
            .addMediaSize(PrintAttributes.MediaSize.ISO_A4, true)
            .addMediaSize(PrintAttributes.MediaSize.NA_LETTER, false)
            .addResolution(PrintAttributes.Resolution("300", "300 dpi", 300, 300), true)
            .setColorModes(
                PrintAttributes.COLOR_MODE_COLOR or PrintAttributes.COLOR_MODE_MONOCHROME,
                PrintAttributes.COLOR_MODE_COLOR,
            )
            .setMinMargins(PrintAttributes.Margins.NO_MARGINS)
            .build()
        return PrinterInfo.Builder(id, PRINTER_NAME, PrinterInfo.STATUS_IDLE)
            .setCapabilities(capabilities)
            .build()
    }

    override fun onPrintJobQueued(printJob: PrintJob) {
        printJob.start()
        try {
            val data = printJob.document.data
                ?: throw IllegalStateException("the print job holds no document")
            val label = printJob.info.label
                .replace(Regex("[/\\\\:*?\"<>|\\p{Cntrl}]"), " ")
                .trim()
                .take(80)
                .ifEmpty { "Print" }
            val target = Captures.free(PrintInbox.dir(this), "$label.pdf")
            FileInputStream(data.fileDescriptor).use { input ->
                target.outputStream().use { output -> input.copyTo(output) }
            }
            printJob.complete()
        } catch (e: Exception) {
            printJob.fail(e.message)
        }
    }

    override fun onRequestCancelPrintJob(printJob: PrintJob) {
        printJob.cancel()
    }

    companion object {
        /** The app's name, which is never translated. */
        const val PRINTER_NAME = "IIbrain"
    }
}

/** Where printed PDFs wait until the app takes them into the Brain. */
object PrintInbox {
    fun dir(context: Context): File =
        File(context.filesDir, "prints").apply { mkdirs() }

    /** The waiting PDFs, oldest first. */
    fun waiting(context: Context): List<String> =
        dir(context).listFiles().orEmpty()
            .filter { it.isFile && it.length() > 0 }
            .sortedBy { it.lastModified() }
            .map { it.absolutePath }
}
