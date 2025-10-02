from playwright.sync_api import Page, expect

def test_realtime_updates(page: Page):
    """
    This test verifies that the WASM UI receives and renders the initial
    game state upon connecting to the WebSocket server. This demonstrates
    that the real-time update mechanism is functioning correctly.
    """
    # 1. Arrange: Go to the application's homepage.
    page.goto("http://127.0.0.1:3000")

    # 2. Act: Navigate to the Poker Online screen.
    poker_online_link = page.get_by_role("link", name="Poker Online")
    poker_online_link.click()

    # 3. Assert: Verify we are on the Poker Online screen.
    expect(page.get_by_role("heading", name="Poker Online")).to_be_visible()

    # 4. Act: Click the "Connect" button within the "Connection & session" collapsible header.
    # The header is open by default.
    connect_button = page.get_by_role("button", name="Connect")
    connect_button.click()

    # 5. Assert: Wait for and verify that the game state has been received and rendered.
    # A key indicator is the appearance of the player table and action buttons.
    # We'll wait for the "Player" column header to appear in the game view.
    expect(page.get_by_role("cell", name="Player")).to_be_visible(timeout=10000)

    # Also check for the pot, which should be visible.
    expect(page.get_by_text("Pot: ")).to_be_visible()

    # 6. Screenshot: Capture the final result for visual verification.
    page.screenshot(path="jules-scratch/verification/verification.png")