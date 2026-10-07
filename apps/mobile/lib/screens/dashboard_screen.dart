import 'package:flutter/material.dart';
import '../models/commitment.dart';
import '../models/status_data.dart';
import '../services/forgex_api_service.dart';

class DashboardScreen extends StatefulWidget {
  final ForgeXApiService apiService;

  const DashboardScreen({super.key, required this.apiService});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  StatusData? _status;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadStatus();
  }

  Future<void> _loadStatus() async {
    setState(() => _isLoading = true);
    final data = await widget.apiService.getStatus();
    if (mounted) {
      setState(() {
        _status = data;
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

    final status = _status!;

    return Scaffold(
      appBar: AppBar(
        title: const Row(
          children: [
            Text(
              'FORGE',
              style: TextStyle(fontWeight: FontWeight.w900, letterSpacing: 2),
            ),
            Text(
              'X',
              style: TextStyle(
                fontWeight: FontWeight.w900,
                color: Colors.cyan,
                letterSpacing: 2,
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadStatus,
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _loadStatus,
        color: Colors.cyan,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _buildSectionHeader('TODAY'),
            if (status.todayCommitments.isEmpty)
              const Padding(
                padding: EdgeInsets.symmetric(vertical: 12),
                child: Text(
                  'No commitments scheduled for today.',
                  style: TextStyle(color: Colors.grey),
                ),
              )
            else
              ...status.todayCommitments.map((c) => _buildCommitmentTile(c)),
            const SizedBox(height: 24),
            _buildSectionHeader('BEHAVIOR'),
            _buildMetricCard(
              title: 'Skip streak',
              value: '${status.skipStreak}',
              color: status.skipStreak > 0 ? Colors.redAccent : Colors.greenAccent,
              subtitle: status.skipStreak > 0
                  ? 'Avoid compounding skips'
                  : 'Clean execution',
            ),
            _buildMetricCard(
              title: 'Entertainment today',
              value: '${status.todayEntertainmentMins}m',
              color: status.todayEntertainmentMins > 60
                  ? Colors.amberAccent
                  : Colors.cyanAccent,
              subtitle: 'Monitored across phone & laptop',
            ),
            _buildMetricCard(
              title: 'Avoidance score',
              value: '${status.avoidanceScore}',
              color: status.avoidanceScore > 50
                  ? Colors.redAccent
                  : (status.avoidanceScore > 0 ? Colors.amberAccent : Colors.greenAccent),
              subtitle: 'Temporal correlation with tasks',
            ),
            const SizedBox(height: 24),
            _buildSectionHeader('PENALTY'),
            _buildPenaltyCard(status),
            const SizedBox(height: 24),
            _buildSectionHeader('SAVE DAYS'),
            _buildSaveDayCard(status),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.bold,
              color: Colors.cyan,
              letterSpacing: 1.5,
            ),
          ),
          const Divider(color: Colors.white24, height: 8),
        ],
      ),
    );
  }

  Widget _buildCommitmentTile(Commitment c) {
    IconData icon;
    Color iconColor;

    if (c.isCompleted) {
      icon = Icons.check_circle;
      iconColor = Colors.greenAccent;
    } else if (c.isMissed) {
      icon = Icons.cancel;
      iconColor = Colors.redAccent;
    } else if (c.isActive) {
      icon = Icons.play_circle_filled;
      iconColor = Colors.amberAccent;
    } else {
      icon = Icons.radio_button_unchecked;
      iconColor = Colors.grey;
    }

    return Card(
      color: const Color(0xFF1E1E24),
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: ListTile(
        leading: Icon(icon, color: iconColor),
        title: Text(
          c.title,
          style: TextStyle(
            fontWeight: FontWeight.w600,
            decoration: c.isCompleted ? TextDecoration.lineThrough : null,
          ),
        ),
        subtitle: Text(
          '${c.category} • ${c.scheduledDurationMins}m • Imp: ${c.importance}/5',
          style: const TextStyle(fontSize: 12, color: Colors.grey),
        ),
        trailing: c.isPlanned
            ? Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  IconButton(
                    icon: const Icon(Icons.check, color: Colors.greenAccent),
                    onPressed: () async {
                      await widget.apiService.completeCommitment(c.id);
                      _loadStatus();
                    },
                  ),
                  IconButton(
                    icon: const Icon(Icons.close, color: Colors.redAccent),
                    onPressed: () async {
                      await widget.apiService.missCommitment(c.id);
                      _loadStatus();
                    },
                  ),
                ],
              )
            : null,
      ),
    );
  }

  Widget _buildMetricCard({
    required String title,
    required String value,
    required Color color,
    required String subtitle,
  }) {
    return Card(
      color: const Color(0xFF1E1E24),
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: const TextStyle(fontWeight: FontWeight.w500)),
                Text(subtitle, style: const TextStyle(fontSize: 11, color: Colors.grey)),
              ],
            ),
            Text(
              value,
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPenaltyCard(StatusData status) {
    final hasPenalty =
        status.remainingPenaltyMins > 0 || status.restrictedDays > 0;

    return Card(
      color: hasPenalty ? const Color(0xFF2C1E20) : const Color(0xFF1E1E24),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Remaining Restriction:'),
                Text(
                  hasPenalty
                      ? '${status.remainingPenaltyMins}m'
                      : '0m (Freedom)',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: hasPenalty ? Colors.redAccent : Colors.greenAccent,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Restricted Days:'),
                Text(
                  '${status.restrictedDays}',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: status.restrictedDays > 0
                        ? Colors.redAccent
                        : Colors.greenAccent,
                  ),
                ),
              ],
            ),
            if (hasPenalty) ...[
              const SizedBox(height: 12),
              const Text(
                'Complete scheduled tasks to earn recovery points and reduce this penalty.',
                style: TextStyle(fontSize: 12, fontStyle: FontStyle.italic, color: Colors.grey),
              ),
            ]
          ],
        ),
      ),
    );
  }

  Widget _buildSaveDayCard(StatusData status) {
    return Card(
      color: const Color(0xFF1E1E24),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Row(
                  children: [
                    Icon(Icons.shield, color: Colors.amberAccent, size: 20),
                    SizedBox(width: 8),
                    Text('Available Save Days:'),
                  ],
                ),
                Text(
                  '${status.availableSaveDays}',
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                    color: Colors.amberAccent,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Weekly Consistency:'),
                Text(
                  status.weeklyConsistency,
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
