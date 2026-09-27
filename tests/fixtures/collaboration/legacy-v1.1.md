# Agent: reviewer

## Metadata

id: urn:uar:legacy:reviewer
version: 1.0.0

## Identity

name: reviewer
role: reviewer
persona: C03 legacy compatibility fixture.

## UI (A2UI)

forms: []

## Capabilities

streaming: true

## Skills

- id: urn:uar:skill:review
  version: 2.1.0

## Tools

allow: [read-artifact]

## MCP Servers

servers: []

## Knowledge Base

sources: []

## Memory Model

conversation: false

## A2A Contracts

contracts: []

## Governance

policies: []

## Budgets & Constraints

max_tokens: 10000

## Execution Model

mode: ordinary-agent

## Observability

audit: required

## Deployment Profiles

profiles: [default]
