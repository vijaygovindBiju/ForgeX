import 'package:flutter_test/flutter_test.dart';
import 'package:forgex_mobile/main.dart';
import 'package:forgex_mobile/services/forgex_api_service.dart';
import 'package:forgex_mobile/services/forgex_ffi_service.dart';

void main() {
  testWidgets('ForgeXApp smoke test', (WidgetTester tester) async {
    final apiService = ForgeXApiService();
    final ffiService = ForgeXFfiService();

    await tester.pumpWidget(ForgeXApp(
      apiService: apiService,
      ffiService: ffiService,
    ));

    // Wait for async load to settle
    await tester.pumpAndSettle();

    expect(find.text('FORGE'), findsOneWidget);
    expect(find.text('X'), findsOneWidget);
    expect(find.text('TODAY'), findsOneWidget);
    expect(find.text('BEHAVIOR'), findsOneWidget);

    // Scroll to see PENALTY and SAVE DAYS
    await tester.scrollUntilVisible(find.text('PENALTY'), 100);
    expect(find.text('PENALTY'), findsOneWidget);

    await tester.scrollUntilVisible(find.text('SAVE DAYS'), 100);
    expect(find.text('SAVE DAYS'), findsOneWidget);
  });
}
