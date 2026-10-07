import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';
import '../models/commitment.dart';
import '../models/character_profile.dart';
import '../models/status_data.dart';

class ForgeXApiService {
  static const String _keyServerUrl = 'forgex_server_url';
  static const String _keyPairingToken = 'forgex_pairing_token';

  String _serverUrl = 'http://10.0.2.2:8080'; // Default Android emulator host loopback
  String _pairingToken = '';

  String get serverUrl => _serverUrl;
  String get pairingToken => _pairingToken;

  Future<void> init() async {
    final prefs = await SharedPreferences.getInstance();
    _serverUrl = prefs.getString(_keyServerUrl) ?? 'http://10.0.2.2:8080';
    _pairingToken = prefs.getString(_keyPairingToken) ?? '';
  }

  Future<void> saveSettings(String url, String token) async {
    final prefs = await SharedPreferences.getInstance();
    _serverUrl = url.trim().replaceAll(RegExp(r'/+$'), '');
    _pairingToken = token.trim();
    await prefs.setString(_keyServerUrl, _serverUrl);
    await prefs.setString(_keyPairingToken, _pairingToken);
  }

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
        if (_pairingToken.isNotEmpty) 'Authorization': 'Bearer $_pairingToken',
      };

  Future<bool> testConnection() async {
    try {
      final res = await http
          .get(Uri.parse('$_serverUrl/api/health'))
          .timeout(const Duration(seconds: 4));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  Future<StatusData> getStatus() async {
    try {
      final res = await http.get(
        Uri.parse('$_serverUrl/api/status'),
        headers: _headers,
      );
      if (res.statusCode == 200) {
        return StatusData.fromJson(jsonDecode(res.body));
      }
    } catch (_) {}

    // Fallback offline mock data if server is unreachable
    return StatusData(
      todayCommitments: [
        Commitment(
          id: 'demo-1',
          title: 'Morning Workout',
          importance: 4,
          expectedEffort: 4,
          scheduledStart: DateTime.now().subtract(const Duration(hours: 1)),
          scheduledDurationMins: 45,
          recurrence: 'daily',
          category: 'Health',
          status: 'planned',
        ),
        Commitment(
          id: 'demo-2',
          title: 'Deep Work: Core Architecture',
          importance: 5,
          expectedEffort: 4,
          scheduledStart: DateTime.now().add(const Duration(hours: 1)),
          scheduledDurationMins: 90,
          recurrence: 'daily',
          category: 'Coding',
          status: 'planned',
        ),
      ],
      skipStreak: 0,
      todayEntertainmentMins: 0,
      avoidanceScore: 0,
      remainingPenaltyMins: 0,
      restrictedDays: 0,
      availableSaveDays: 1,
      weeklyConsistency: '6/7',
    );
  }

  Future<List<Commitment>> getCommitments() async {
    try {
      final res = await http.get(
        Uri.parse('$_serverUrl/api/commitments'),
        headers: _headers,
      );
      if (res.statusCode == 200) {
        final list = jsonDecode(res.body) as List<dynamic>;
        return list.map((e) => Commitment.fromJson(e)).toList();
      }
    } catch (_) {}
    return [];
  }

  Future<bool> createCommitment({
    required String title,
    required DateTime start,
    required int duration,
    required int importance,
    required int effort,
    required String category,
  }) async {
    try {
      final res = await http.post(
        Uri.parse('$_serverUrl/api/commitments'),
        headers: _headers,
        body: jsonEncode({
          'title': title,
          'start': start.toUtc().toIso8601String(),
          'duration': duration,
          'importance': importance,
          'effort': effort,
          'category': category,
        }),
      );
      return res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  Future<bool> startCommitment(String id) async {
    try {
      final res = await http.post(
        Uri.parse('$_serverUrl/api/commitments/$id/start'),
        headers: _headers,
      );
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  Future<Map<String, dynamic>?> completeCommitment(String id) async {
    try {
      final res = await http.post(
        Uri.parse('$_serverUrl/api/commitments/$id/complete'),
        headers: _headers,
      );
      if (res.statusCode == 200) {
        return jsonDecode(res.body);
      }
    } catch (_) {}
    return null;
  }

  Future<Map<String, dynamic>?> missCommitment(String id, {bool useSaveDay = false}) async {
    try {
      final res = await http.post(
        Uri.parse('$_serverUrl/api/commitments/$id/miss'),
        headers: _headers,
        body: jsonEncode({'force_use_save_day': useSaveDay}),
      );
      if (res.statusCode == 200) {
        return jsonDecode(res.body);
      }
    } catch (_) {}
    return null;
  }

  Future<bool> logActivity({
    required String application,
    String? detail,
    required int durationMins,
    required String category,
  }) async {
    try {
      final res = await http.post(
        Uri.parse('$_serverUrl/api/activities'),
        headers: _headers,
        body: jsonEncode({
          'application': application,
          'detail': detail,
          'duration_mins': durationMins,
          'category': category,
          'device': 'android-mobile',
        }),
      );
      return res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  Future<CharacterProfile> getCharacterProfile() async {
    try {
      final res = await http.get(
        Uri.parse('$_serverUrl/api/character'),
        headers: _headers,
      );
      if (res.statusCode == 200) {
        return CharacterProfile.fromJson(jsonDecode(res.body));
      }
    } catch (_) {}
    return CharacterProfile(
      consistency: 78,
      reliability: 85,
      resilience: 70,
      selfControl: 80,
      discipline: 75,
      overallLevel: 7,
    );
  }
}
