import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';
import 'note_page.dart';

/// The search: words in, the Notes that hold them out.
class SearchTab extends StatefulWidget {
  const SearchTab({super.key});

  @override
  State<SearchTab> createState() => _SearchTabState();
}

class _SearchTabState extends State<SearchTab> {
  final _query = TextEditingController();
  List<Hit>? _hits;

  /// Counts the searches, so that an answer that comes late does not
  /// replace a later one.
  int _asked = 0;

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  Future<void> _search(String query) async {
    final asked = ++_asked;
    if (query.trim().isEmpty) {
      setState(() => _hits = null);
      return;
    }
    try {
      final hits = await AppScope.of(context).core.search(query);
      if (!mounted || asked != _asked) return;
      setState(() => _hits = hits);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _open(Hit hit) async {
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (context) => NotePage(id: hit.id)));
    await _search(_query.text);
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final hits = _hits;
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            controller: _query,
            decoration: InputDecoration(
              labelText: t.search,
              prefixIcon: const Icon(Icons.search),
              border: const OutlineInputBorder(),
            ),
            onChanged: (query) => unawaited(_search(query)),
          ),
        ),
        Expanded(
          child: hits == null
              ? Center(child: Text(t.searchHint))
              : ListView(
                  children: [
                    ListTile(title: Text(t.searchResults(hits.length))),
                    for (final hit in hits)
                      ListTile(
                        title: Text(hit.title),
                        subtitle: Text(hit.excerpt),
                        onTap: () => unawaited(_open(hit)),
                      ),
                  ],
                ),
        ),
      ],
    );
  }
}
