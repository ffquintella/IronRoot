Feature: Startup banner

  The placeholder binary prints a banner and exits. Until `ironroot-web` is
  wired in, the banner has to describe what is still missing without implying
  that an HTTP server came up.

  Scenario: The banner introduces the application
    When the startup banner is rendered
    Then the banner should start with "IronRoot Web Application"
    And the banner should contain "templates/web-app/README.md"

  Scenario: The banner does not claim a running server
    When the startup banner is rendered
    Then the banner should not contain "listening on"
    And the banner should not contain "server started"
