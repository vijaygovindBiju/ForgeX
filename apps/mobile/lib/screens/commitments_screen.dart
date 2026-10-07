import 'package:flutter/material.dart';
import '../models/commitment.dart';
import '../services/forgex_api_service.dart';

class CommitmentsScreen extends StatefulWidget {
  final ForgeXApiService apiService;

  const CommitmentsScreen({super.key, required this.apiService});

  @override
  State<CommitmentsScreen> createState() => _CommitmentsScreenState();
}

class _CommitmentsScreenState extends State<CommitmentsScreen> {
  List<Commitment> _commitments = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadCommitments();
  }

  Future<void> _loadCommitments() async {
    setState(() => _isLoading = true);
    final list = await widget.apiService.getCommitments();
    if (mounted) {
      setState(() {
        _commitments = list;
        _isLoading = false;
      });
    }
  }

  void _showAddDialog() {
    final titleCtrl = TextEditingController();
    final durationCtrl = TextEditingController(text: '45');
    final catCtrl = TextEditingController(text: 'General');
    int importance = 3;
    int effort = 3;

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          backgroundColor: const Color(0xFF1E1E24),
          title: const Text('New Commitment'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: titleCtrl,
                  decoration: const InputDecoration(labelText: 'Title'),
                ),
                TextField(
                  controller: durationCtrl,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(labelText: 'Duration (mins)'),
                ),
                TextField(
                  controller: catCtrl,
                  decoration: const InputDecoration(labelText: 'Category'),
                ),
                const SizedBox(height: 16),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Importance (1-5):'),
                    DropdownButton<int>(
                      value: importance,
                      dropdownColor: const Color(0xFF1E1E24),
                      items: [1, 2, 3, 4, 5]
                          .map((i) => DropdownMenuItem(value: i, child: Text('$i')))
                          .toList(),
                      onChanged: (v) => setDialogState(() => importance = v ?? 3),
                    ),
                  ],
                ),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Effort (1-5):'),
                    DropdownButton<int>(
                      value: effort,
                      dropdownColor: const Color(0xFF1E1E24),
                      items: [1, 2, 3, 4, 5]
                          .map((i) => DropdownMenuItem(value: i, child: Text('$i')))
                          .toList(),
                      onChanged: (v) => setDialogState(() => effort = v ?? 3),
                    ),
                  ],
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () async {
                if (titleCtrl.text.trim().isNotEmpty) {
                  final dur = int.tryParse(durationCtrl.text) ?? 45;
                  await widget.apiService.createCommitment(
                    title: titleCtrl.text.trim(),
                    start: DateTime.now(),
                    duration: dur,
                    importance: importance,
                    effort: effort,
                    category: catCtrl.text.trim(),
                  );
                  if (ctx.mounted) {
                    Navigator.pop(ctx);
                  }
                  _loadCommitments();
                }
              },
              child: const Text('Create'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Commitments'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadCommitments,
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        backgroundColor: Colors.cyan,
        foregroundColor: Colors.black,
        onPressed: _showAddDialog,
        child: const Icon(Icons.add),
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator(color: Colors.cyan))
          : _commitments.isEmpty
              ? const Center(
                  child: Text(
                    'No commitments found.\nTap + to create a promise to yourself.',
                    textAlign: TextAlign.center,
                    style: TextStyle(color: Colors.grey),
                  ),
                )
              : ListView.builder(
                  padding: const EdgeInsets.all(16),
                  itemCount: _commitments.length,
                  itemBuilder: (context, index) {
                    final c = _commitments[index];
                    return Card(
                      color: const Color(0xFF1E1E24),
                      margin: const EdgeInsets.symmetric(vertical: 6),
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                Expanded(
                                  child: Text(
                                    c.title,
                                    style: const TextStyle(
                                      fontSize: 16,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                ),
                                Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                  decoration: BoxDecoration(
                                    color: _statusColor(c.status).withValues(alpha: 0.2),
                                    borderRadius: BorderRadius.circular(4),
                                  ),
                                  child: Text(
                                    c.status.toUpperCase(),
                                    style: TextStyle(
                                      fontSize: 11,
                                      fontWeight: FontWeight.bold,
                                      color: _statusColor(c.status),
                                    ),
                                  ),
                                ),
                              ],
                            ),
                            const SizedBox(height: 8),
                            Text(
                              '${c.category} • ${c.scheduledDurationMins}m • Imp: ${c.importance}/5 • Effort: ${c.expectedEffort}/5',
                              style: const TextStyle(fontSize: 12, color: Colors.grey),
                            ),
                            if (c.isPlanned || c.isActive) ...[
                              const Divider(color: Colors.white12, height: 16),
                              Row(
                                mainAxisAlignment: MainAxisAlignment.end,
                                children: [
                                  if (c.isPlanned)
                                    TextButton.icon(
                                      icon: const Icon(Icons.play_arrow, size: 16),
                                      label: const Text('Start'),
                                      onPressed: () async {
                                        await widget.apiService.startCommitment(c.id);
                                        _loadCommitments();
                                      },
                                    ),
                                  TextButton.icon(
                                    icon: const Icon(Icons.check, size: 16, color: Colors.greenAccent),
                                    label: const Text('Keep', style: TextStyle(color: Colors.greenAccent)),
                                    onPressed: () async {
                                      await widget.apiService.completeCommitment(c.id);
                                      _loadCommitments();
                                    },
                                  ),
                                  TextButton.icon(
                                    icon: const Icon(Icons.close, size: 16, color: Colors.redAccent),
                                    label: const Text('Miss', style: TextStyle(color: Colors.redAccent)),
                                    onPressed: () async {
                                      await widget.apiService.missCommitment(c.id);
                                      _loadCommitments();
                                    },
                                  ),
                                ],
                              ),
                            ]
                          ],
                        ),
                      ),
                    );
                  },
                ),
    );
  }

  Color _statusColor(String status) {
    switch (status.toLowerCase()) {
      case 'completed':
        return Colors.greenAccent;
      case 'missed':
        return Colors.redAccent;
      case 'active':
        return Colors.amberAccent;
      default:
        return Colors.cyan;
    }
  }
}
