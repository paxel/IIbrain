package io.github.paxel.iibrain

import android.content.ContentUris
import android.content.Context
import android.provider.CalendarContract
import android.provider.ContactsContract
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Live Lookups (docs/adr/0010): the calendar and the contacts, read at
 * the moment a Question needs them and handed over as text. Nothing of
 * them is kept anywhere.
 */
object Lookups {
    private const val DAYS = 14
    private const val MOST = 40

    /** The events of the next fourteen days, one per line. */
    fun calendar(context: Context): String {
        val begin = System.currentTimeMillis()
        val end = begin + DAYS * 24L * 60 * 60 * 1000
        val uri = CalendarContract.Instances.CONTENT_URI.buildUpon().let {
            ContentUris.appendId(it, begin)
            ContentUris.appendId(it, end)
            it.build()
        }
        val day = SimpleDateFormat("yyyy-MM-dd", Locale.US)
        val minute = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US)
        val hour = SimpleDateFormat("HH:mm", Locale.US)
        val lines = mutableListOf<String>()
        context.contentResolver.query(
            uri,
            arrayOf(
                CalendarContract.Instances.TITLE,
                CalendarContract.Instances.BEGIN,
                CalendarContract.Instances.END,
                CalendarContract.Instances.ALL_DAY,
                CalendarContract.Instances.EVENT_LOCATION,
            ),
            null,
            null,
            CalendarContract.Instances.BEGIN + " ASC",
        )?.use { cursor ->
            while (cursor.moveToNext() && lines.size < MOST) {
                val title = cursor.getString(0).orEmpty().ifBlank { "(no title)" }
                val from = Date(cursor.getLong(1))
                val to = Date(cursor.getLong(2))
                val time = if (cursor.getInt(3) != 0) {
                    day.format(from)
                } else {
                    "${minute.format(from)} to ${hour.format(to)}"
                }
                val place = cursor.getString(4).orEmpty()
                lines += if (place.isBlank()) "$time: $title" else "$time: $title ($place)"
            }
        }
        return lines.joinToString("\n")
    }

    /**
     * The contacts whose names share a word with [words], each with its
     * phone numbers and mail addresses, one per line.
     */
    fun contacts(context: Context, words: List<String>): String {
        val found = linkedMapOf<Long, String>()
        for (word in words.filter { it.length >= 3 }) {
            context.contentResolver.query(
                ContactsContract.Contacts.CONTENT_URI,
                arrayOf(
                    ContactsContract.Contacts._ID,
                    ContactsContract.Contacts.DISPLAY_NAME_PRIMARY,
                ),
                ContactsContract.Contacts.DISPLAY_NAME_PRIMARY + " LIKE ?",
                arrayOf("%$word%"),
                null,
            )?.use { cursor ->
                while (cursor.moveToNext() && found.size < MOST) {
                    found[cursor.getLong(0)] = cursor.getString(1).orEmpty()
                }
            }
        }
        return found.entries.joinToString("\n") { (id, name) ->
            val reach = values(
                context,
                ContactsContract.CommonDataKinds.Phone.CONTENT_URI,
                ContactsContract.CommonDataKinds.Phone.CONTACT_ID,
                ContactsContract.CommonDataKinds.Phone.NUMBER,
                id,
            ) + values(
                context,
                ContactsContract.CommonDataKinds.Email.CONTENT_URI,
                ContactsContract.CommonDataKinds.Email.CONTACT_ID,
                ContactsContract.CommonDataKinds.Email.ADDRESS,
                id,
            )
            if (reach.isEmpty()) name else "$name: ${reach.joinToString(", ")}"
        }
    }

    private fun values(
        context: Context,
        uri: android.net.Uri,
        idColumn: String,
        valueColumn: String,
        id: Long,
    ): List<String> {
        val out = mutableListOf<String>()
        context.contentResolver.query(
            uri,
            arrayOf(valueColumn),
            "$idColumn = ?",
            arrayOf(id.toString()),
            null,
        )?.use { cursor ->
            while (cursor.moveToNext()) {
                cursor.getString(0)?.takeIf { it.isNotBlank() }?.let { out += it }
            }
        }
        return out.distinct()
    }
}
