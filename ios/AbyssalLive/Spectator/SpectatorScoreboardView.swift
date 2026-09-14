import SwiftUI

/// Read-only live view for parents/spectators - re-implements what
/// `frontend/scoreboard.js` shows on the venue's TV, driven by
/// `ScoreboardSocket` instead of a poll loop of its own.
struct SpectatorScoreboardView: View {
    @EnvironmentObject private var session: ServerSession
    let competition: Competition
    let onChangeCompetition: () -> Void

    @StateObject private var socket = ScoreboardSocket()
    @State private var events: [MeetEvent] = []
    @State private var participants: [Participant] = []

    var body: some View {
        VStack(spacing: 24) {
            Text(competition.name)
                .font(.title2.bold())
                .multilineTextAlignment(.center)

            Spacer()

            if let state = socket.state {
                VStack(spacing: 12) {
                    Text(currentEventName(state))
                        .font(.title3)
                        .foregroundStyle(.secondary)
                    Text(currentParticipantName(state))
                        .font(.system(size: 44, weight: .bold, design: .rounded))
                        .multilineTextAlignment(.center)
                    Text(state.displayMode.label)
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }
            } else {
                ProgressView("Connecting...")
            }

            Spacer()

            if !socket.isConnected {
                Label("Reconnecting...", systemImage: "wifi.slash")
                    .foregroundStyle(.orange)
            }
        }
        .padding()
        .navigationTitle("Live")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Change competition", action: onChangeCompetition)
            }
        }
        .task {
            socket.connect(session: session, competitionId: competition.id)
            await loadNames()
        }
        .onDisappear { socket.disconnect() }
    }

    private func loadNames() async {
        let client = APIClient(session: session)
        events = (try? await client.listEvents(competitionId: competition.id)) ?? []
        participants = (try? await client.listParticipants(competitionId: competition.id)) ?? []
    }

    private func currentParticipantName(_ state: ScoreboardState) -> String {
        guard let id = state.currentParticipantId,
              let participant = participants.first(where: { $0.id == id })
        else { return "-" }
        return participant.fullName
    }

    private func currentEventName(_ state: ScoreboardState) -> String {
        guard let id = state.currentEventId,
              let event = events.first(where: { $0.id == id })
        else { return " " }
        return event.name
    }
}

#Preview {
    NavigationStack {
        SpectatorScoreboardView(
            competition: Competition(id: UUID(), name: "Fall Invitational", sport: "gymnastics", date: "2026-09-09", location: nil, themeId: nil),
            onChangeCompetition: {}
        )
        .environmentObject(ServerSession())
    }
}
