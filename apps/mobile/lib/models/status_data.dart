import 'commitment.dart';

class StatusData {
  final List<Commitment> todayCommitments;
  final int skipStreak;
  final int todayEntertainmentMins;
  final int avoidanceScore;
  final int remainingPenaltyMins;
  final int restrictedDays;
  final int availableSaveDays;
  final String weeklyConsistency;

  StatusData({
    required this.todayCommitments,
    required this.skipStreak,
    required this.todayEntertainmentMins,
    required this.avoidanceScore,
    required this.remainingPenaltyMins,
    required this.restrictedDays,
    required this.availableSaveDays,
    required this.weeklyConsistency,
  });

  factory StatusData.fromJson(Map<String, dynamic> json) {
    final list = (json['today_commitments'] as List<dynamic>?)
            ?.map((e) => Commitment.fromJson(e as Map<String, dynamic>))
            .toList() ??
        [];

    final penalty = json['penalty'] as Map<String, dynamic>? ?? {};
    final saveDays = json['save_days'] as Map<String, dynamic>? ?? {};

    return StatusData(
      todayCommitments: list,
      skipStreak: json['skip_streak'] ?? 0,
      todayEntertainmentMins: json['today_entertainment_mins'] ?? 0,
      avoidanceScore: json['avoidance_score'] ?? 0,
      remainingPenaltyMins: penalty['remaining_mins'] ?? 0,
      restrictedDays: penalty['restricted_days'] ?? 0,
      availableSaveDays: saveDays['available'] ?? 0,
      weeklyConsistency: saveDays['weekly_consistency'] ?? '0/7',
    );
  }
}
