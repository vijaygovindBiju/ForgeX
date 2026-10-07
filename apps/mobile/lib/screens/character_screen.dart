import 'package:flutter/material.dart';
import '../models/character_profile.dart';
import '../services/forgex_api_service.dart';

class CharacterScreen extends StatefulWidget {
  final ForgeXApiService apiService;

  const CharacterScreen({super.key, required this.apiService});

  @override
  State<CharacterScreen> createState() => _CharacterScreenState();
}

class _CharacterScreenState extends State<CharacterScreen> {
  CharacterProfile? _profile;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadProfile();
  }

  Future<void> _loadProfile() async {
    setState(() => _isLoading = true);
    final p = await widget.apiService.getCharacterProfile();
    if (mounted) {
      setState(() {
        _profile = p;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator(color: Colors.cyan)),
      );
    }

    final p = _profile!;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Character Profile'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadProfile,
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            color: const Color(0xFF1E1E24),
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                children: [
                  const Text('CHARACTER MASTERY', style: TextStyle(fontSize: 12, letterSpacing: 2, color: Colors.grey)),
                  const SizedBox(height: 8),
                  Text(
                    'Level ${p.overallLevel}',
                    style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w900, color: Colors.cyan),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Overall Average: ${p.averageScore}%',
                    style: const TextStyle(fontSize: 14, color: Colors.white70),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 20),
          _buildMetricRow('Consistency', p.consistency, 'How often promises are fulfilled'),
          _buildMetricRow('Reliability', p.reliability, 'Follow-through without prior avoidance'),
          _buildMetricRow('Resilience', p.resilience, 'Recovery speed following missed tasks'),
          _buildMetricRow('Self-Control', p.selfControl, 'Absence of entertainment during tasks'),
          _buildMetricRow('Discipline', p.discipline, 'Execution on high-effort & high-importance commitments'),
          const SizedBox(height: 24),
          const Center(
            child: Text(
              'These metrics reflect observed behavior, not your human worth.\nYou forge character through repeated follow-through.',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12, fontStyle: FontStyle.italic, color: Colors.grey),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMetricRow(String name, int score, String desc) {
    final color = score >= 80 ? Colors.greenAccent : (score >= 60 ? Colors.amberAccent : Colors.redAccent);

    return Card(
      color: const Color(0xFF1E1E24),
      margin: const EdgeInsets.symmetric(vertical: 6),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(name, style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 15)),
                Text('$score%', style: TextStyle(fontWeight: FontWeight.bold, color: color, fontSize: 16)),
              ],
            ),
            const SizedBox(height: 8),
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: score / 100.0,
                backgroundColor: Colors.white10,
                valueColor: AlwaysStoppedAnimation<Color>(color),
                minHeight: 8,
              ),
            ),
            const SizedBox(height: 6),
            Text(desc, style: const TextStyle(fontSize: 11, color: Colors.grey)),
          ],
        ),
      ),
    );
  }
}
