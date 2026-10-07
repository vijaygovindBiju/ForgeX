class Commitment {
  final String id;
  final String title;
  final String? description;
  final int importance;
  final int expectedEffort;
  final DateTime scheduledStart;
  final int scheduledDurationMins;
  final String recurrence;
  final String category;
  final String status;
  final int consecutiveSkips;

  Commitment({
    required this.id,
    required this.title,
    this.description,
    required this.importance,
    required this.expectedEffort,
    required this.scheduledStart,
    required this.scheduledDurationMins,
    required this.recurrence,
    required this.category,
    required this.status,
    this.consecutiveSkips = 0,
  });

  factory Commitment.fromJson(Map<String, dynamic> json) {
    return Commitment(
      id: json['id'] ?? '',
      title: json['title'] ?? '',
      description: json['description'],
      importance: json['importance'] ?? 3,
      expectedEffort: json['expected_effort'] ?? 3,
      scheduledStart: DateTime.parse(json['scheduled_start']),
      scheduledDurationMins: json['scheduled_duration_mins'] ?? 45,
      recurrence: json['recurrence'] is String
          ? json['recurrence']
          : (json['recurrence']?['type'] ?? 'daily'),
      category: json['category'] ?? 'General',
      status: json['status'] ?? 'planned',
      consecutiveSkips: json['consecutive_skips'] ?? 0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'title': title,
      'description': description,
      'importance': importance,
      'expected_effort': expectedEffort,
      'scheduled_start': scheduledStart.toIso8601String(),
      'scheduled_duration_mins': scheduledDurationMins,
      'recurrence': recurrence,
      'category': category,
      'status': status,
      'consecutive_skips': consecutiveSkips,
    };
  }

  bool get isCompleted => status.toLowerCase() == 'completed';
  bool get isMissed => status.toLowerCase() == 'missed';
  bool get isActive => status.toLowerCase() == 'active';
  bool get isPlanned => status.toLowerCase() == 'planned';
}
