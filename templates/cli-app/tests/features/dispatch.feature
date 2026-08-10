Feature: Placeholder command dispatch

  The binary has no commands yet. Until `ironroot-cli` is wired in it either
  explains how it should be invoked, or acknowledges the arguments it was given
  — never both, and never a claim that a command actually ran.

  Scenario: An invocation with arguments reports what would be dispatched
    Given the command line arguments "greet Alice"
    When the output is rendered
    Then the output should contain "Arguments received"
    And the output should contain "Alice"
    And the output should contain "Dispatch to the matching Command"

  Scenario: An invocation with no arguments prints usage instead
    Given no command line arguments
    When the output is rendered
    Then the output should contain "Usage: ironroot-cli-app"
    And the output should contain "greet <name>"
    And the output should not contain "Arguments received"

  Scenario: No invocation claims a command was executed
    Given the command line arguments "greet Alice"
    When the output is rendered
    Then the output should not contain "command executed"
