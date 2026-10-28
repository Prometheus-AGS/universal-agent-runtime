@api @streaming @agui
Feature: Legacy AG-UI and dual chat stream framing
  Scenario: The legacy AG-UI chat stream answers and continues a session
    Given a disposable saved agent with a private instruction marker for chat streaming
    When I stream a fresh calculation and then a dependent follow-up with stream_mode agui on /api/chat/completion
    Then each legacy AG-UI stream delivers incremental text, completes, and answers correctly in one session

  Scenario: The dual chat stream carries both vocabularies for one run without doubling the answer
    Given a disposable saved agent with a private instruction marker for chat streaming
    When I stream a fresh calculation and then a dependent follow-up with stream_mode dual on /v1/chat/completions
    Then each dual stream's OpenAI and AG-UI text agree, belong to one run, and answer correctly in one session
