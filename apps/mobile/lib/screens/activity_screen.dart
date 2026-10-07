import 'package:flutter/material.dart';
import '../services/forgex_api_service.dart';

class ActivityScreen extends StatefulWidget {
  final ForgeXApiService apiService;

  const ActivityScreen({super.key, required this.apiService});

  @override
  State<ActivityScreen> createState() => _ActivityScreenState();
}

class _ActivityScreenState extends State<ActivityScreen> {
  final _appCtrl = TextEditingController(text: 'YouTube');
  final _detailCtrl = TextEditingController(text: 'Shorts feed');
  final _durationCtrl = TextEditingController(text: '30');
  String _category = 'high';
  bool _isSaving = false;

  final Map<String, String> _categories = {
    'productive': 'Productive (IDE, Study, Docs)',
    'essential': 'Essential (Calls, Maps, Bank)',
    'medium': 'Medium Entertainment (Movies, Reddit)',
    'high': 'High Entertainment (Shorts, Games)',
  };

  Future<void> _submit() async {
    final app = _appCtrl.text.trim();
    final duration = int.tryParse(_durationCtrl.text.trim()) ?? 0;
    if (app.isEmpty || duration <= 0) return;

    setState(() => _isSaving = true);
    final ok = await widget.apiService.logActivity(
      application: app,
      detail: _detailCtrl.text.trim().isNotEmpty ? _detailCtrl.text.trim() : null,
      durationMins: duration,
      category: _category,
    );
    setState(() => _isSaving = false);

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ok ? 'Activity logged successfully' : 'Failed to log activity'),
          backgroundColor: ok ? Colors.green : Colors.redAccent,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Log Activity / Usage')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text(
            'Cross-Device Behavioral Observation',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: Colors.cyan),
          ),
          const SizedBox(height: 8),
          const Text(
            'ForgeX correlates phone entertainment activities with your scheduled tasks to measure avoidance and calibrate consequences.',
            style: TextStyle(fontSize: 13, color: Colors.grey),
          ),
          const SizedBox(height: 24),
          TextField(
            controller: _appCtrl,
            decoration: const InputDecoration(
              labelText: 'Application Name',
              hintText: 'e.g. YouTube, Instagram, VS Code',
              filled: true,
              fillColor: Color(0xFF1E1E24),
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _detailCtrl,
            decoration: const InputDecoration(
              labelText: 'Domain / Content Detail (Optional)',
              hintText: 'e.g. shorts, rust-lang.org',
              filled: true,
              fillColor: Color(0xFF1E1E24),
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _durationCtrl,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
              labelText: 'Duration (Minutes)',
              filled: true,
              fillColor: Color(0xFF1E1E24),
            ),
          ),
          const SizedBox(height: 20),
          const Text('Category:', style: TextStyle(fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          ..._categories.entries.map(
            (e) {
              final isSelected = _category == e.key;
              return Card(
                color: isSelected
                    ? Colors.cyan.withValues(alpha: 0.15)
                    : const Color(0xFF1E1E24),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(8),
                  side: BorderSide(
                    color: isSelected ? Colors.cyan : Colors.transparent,
                  ),
                ),
                margin: const EdgeInsets.symmetric(vertical: 4),
                child: ListTile(
                  dense: true,
                  title: Text(
                    e.value,
                    style: TextStyle(
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                      color: isSelected ? Colors.cyan : Colors.white70,
                    ),
                  ),
                  trailing: isSelected
                      ? const Icon(Icons.check_circle, color: Colors.cyan, size: 20)
                      : null,
                  onTap: () => setState(() => _category = e.key),
                ),
              );
            },
          ),
          const SizedBox(height: 24),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.cyan,
              foregroundColor: Colors.black,
              padding: const EdgeInsets.symmetric(vertical: 14),
            ),
            onPressed: _isSaving ? null : _submit,
            child: _isSaving
                ? const SizedBox(
                    height: 20,
                    width: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('Log Behavioral Activity', style: TextStyle(fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }
}
