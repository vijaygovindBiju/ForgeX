class CharacterProfile {
  final int consistency;
  final int reliability;
  final int resilience;
  final int selfControl;
  final int discipline;
  final int overallLevel;

  CharacterProfile({
    required this.consistency,
    required this.reliability,
    required this.resilience,
    required this.selfControl,
    required this.discipline,
    required this.overallLevel,
  });

  factory CharacterProfile.fromJson(Map<String, dynamic> json) {
    return CharacterProfile(
      consistency: json['consistency'] ?? 50,
      reliability: json['reliability'] ?? 50,
      resilience: json['resilience'] ?? 50,
      selfControl: json['self_control'] ?? 50,
      discipline: json['discipline'] ?? 50,
      overallLevel: json['overall_level'] ?? 1,
    );
  }

  int get averageScore =>
      ((consistency + reliability + resilience + selfControl + discipline) / 5)
          .round();
}
