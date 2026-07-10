# Signal Flow

## Greeting Mode

```mermaid
flowchart LR
  Primary["Primary Site browser"] --> RTC["RTC room: campus-greeting"]
  RemoteA["Remote Site A browser"] --> RTC
  RemoteB["Remote Site B browser"] --> RTC
  Operator["Operator dashboard"] --> RTC
  RTC --> RPrimary["Receiver: /receiver/main"]
  RTC --> RA["Receiver: /receiver/campus-b"]
  RTC --> RB["Receiver: /receiver/campus-c"]
  RTC --> Grid["Receiver: 3-way grid"]
  RPrimary --> Output["Primary Site HDMI / SDI output path"]
```

## Program Mode

```mermaid
flowchart LR
  Program["Primary Site final program"] --> Enc["OBS / hardware encoder / Capture Bridge"]
  Enc -->|"SRT UDP 9999 streamid=default/broadcast/program"| Relay["Streaming relay"]
  Relay -->|"LL-HLS https://stream.example.com/broadcast/program/llhls.m3u8"| Player["Branded program player"]
  Player --> RemoteA["Remote Site A display"]
  Player --> RemoteB["Remote Site B display"]
  PrimaryBrowser["Primary Site browser"] -->|"optional RTC monitor publish"| ProgramRoom["RTC room: campus-program"]
  ProgramRoom --> Operator["Operator monitor"]
```

## Rooms

- `campus-greeting`: all three sites publish and subscribe.
- `campus-program`: Primary Site can publish; remote sites subscribe only.
- `campus-backstage`: reserved for future production workflows.
