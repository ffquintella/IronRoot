Feature: Startup banner

  The placeholder binary prints a banner and exits. Until `ironroot-gui` is
  wired in and a backend feature flag is chosen, the banner has to describe what
  is still missing without implying that a window opened.

  Scenario: The banner introduces the application
    When the startup banner is rendered
    Then the banner should start with "IronRoot Desktop Application"
    And the banner should contain "templates/desktop-app/README.md"

  Scenario: The banner presents both backends as planned, not chosen
    When the startup banner is rendered
    Then the banner should contain "Planned backends"
    And the banner should contain "egui"
    And the banner should contain "tauri"

  Scenario: The banner does not claim an open window
    When the startup banner is rendered
    Then the banner should not contain "window opened"
    And the banner should not contain "backend selected"
