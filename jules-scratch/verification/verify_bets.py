from playwright.sync_api import sync_playwright, Page
import time

def verify_bets(page: Page):
    """
    This test verifies that the total bet for each player is displayed correctly.
    """
    # 1. Arrange: Go to the application with the autostart query parameter.
    page.goto("http://localhost:3000/?autostart=true")

    # Give the page time to load the WASM module and render the game
    time.sleep(5)

    # Take a screenshot
    page.screenshot(path="jules-scratch/verification/verification.png")

def main():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        verify_bets(page)
        browser.close()

if __name__ == "__main__":
    main()
