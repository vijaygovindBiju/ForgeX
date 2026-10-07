class Activity {
  final String id;
  final String device;
  final String application;
  final String? domainOrDetail;
  final String category;
  final DateTime startedAt;
  final int durationMins;

  Activity({
    required this.id,
    required this.device,
    required this.application,
    this.domainOrDetail,
    required this.category,
    required this.startedAt,
    required this.durationMins,
  });

  factory Activity.fromJson(Map<String, dynamic> json) {
    return Activity(
      id: json['id'] ?? '',
      device: json['device'] ?? 'android-phone',
      application: json['application'] ?? '',
      domainOrDetail: json['domain_or_detail'],
      category: json['category'] ?? 'Medium Entertainment',
      startedAt: DateTime.parse(json['started_at']),
      durationMins: json['duration_mins'] ?? 0,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'device': device,
      'application': application,
      'domain_or_detail': domainOrDetail,
      'category': category,
      'started_at': startedAt.toIso8601String(),
      'duration_mins': durationMins,
    };
  }
}
