import SwiftUI

/// Which scoring form to show. The API doesn't expose the
/// competition's sport to a paired device today (`Judge` doesn't
/// carry it), so this is a manual picker rather than auto-detected -
/// the judge already knows which meet they're at. A reasonable follow-up
/// is threading `Competition.sport` through `/pair`'s response.
private enum SportKind: String, CaseIterable, Identifiable {
    case gymnastics
    case timeTrial

    var id: String { rawValue }

    var label: String {
        switch self {
        case .gymnastics: return "Gymnastics"
        case .timeTrial: return "Swimming / Track"
        }
    }
}

struct JudgeHomeView: View {
    @EnvironmentObject private var session: ServerSession
    let onSignOut: () -> Void

    @StateObject private var socket = ScoreboardSocket()
    @State private var sportKind: SportKind = .gymnastics
    @State private var events: [MeetEvent] = []
    @State private var participants: [Participant] = []
    @State private var selectedEventID: UUID?
    @State private var selectedParticipantID: UUID?
    @State private var isLoading = false
    @State private var errorMessage: String?
    @State private var showScoring = false

    private var selectedEvent: MeetEvent? {
        events.first { $0.id == selectedEventID }
    }

    private var selectedParticipant: Participant? {
        participants.first { $0.id == selectedParticipantID }
    }

    var body: some View {
        List {
            if let judge = session.judge {
                Section("Paired as") {
                    Text(judge.name).font(.headline)
                    Text(judge.role.capitalized).foregroundStyle(.secondary)
                }
            }

            if let state = socket.state {
                Section("Now on the board") {
                    Label(state.displayMode.label, systemImage: socket.isConnected ? "dot.radiowaves.left.and.right" : "wifi.slash")
                }
            }

            Section("Sport") {
                Picker("Sport", selection: $sportKind) {
                    ForEach(SportKind.allCases) { kind in
                        Text(kind.label).tag(kind)
                    }
                }
                .pickerStyle(.segmented)
            }

            Section("Event") {
                Picker("Event", selection: $selectedEventID) {
                    Text("Choose an event").tag(UUID?.none)
                    ForEach(events) { event in
                        Text(event.name).tag(Optional(event.id))
                    }
                }
            }

            Section("Participant") {
                Picker("Participant", selection: $selectedParticipantID) {
                    Text("Choose a participant").tag(UUID?.none)
                    ForEach(participants) { participant in
                        Text(participant.fullName).tag(Optional(participant.id))
                    }
                }
            }

            if let errorMessage {
                Section {
                    Text(errorMessage).foregroundStyle(.red)
                }
            }

            Section {
                Button("Score this routine") {
                    showScoring = true
                }
                .disabled(selectedEvent == nil || selectedParticipant == nil)
            }

            Section {
                Button("Sign out", role: .destructive, action: onSignOut)
            }
        }
        .navigationTitle("Judge")
        .task { await load() }
        .onAppear {
            if let competitionId = session.judge?.competitionId {
                socket.connect(session: session, competitionId: competitionId)
            }
        }
        .onDisappear { socket.disconnect() }
        .refreshable { await load() }
        .overlay {
            if isLoading { ProgressView() }
        }
        .sheet(isPresented: $showScoring) {
            if let event = selectedEvent, let participant = selectedParticipant {
                NavigationStack {
                    switch sportKind {
                    case .gymnastics:
                        GymnasticsScoringView(event: event, participant: participant)
                    case .timeTrial:
                        SwimmingScoringView(event: event, participant: participant)
                    }
                }
            }
        }
    }

    private func load() async {
        guard let competitionId = session.judge?.competitionId else { return }
        isLoading = true
        defer { isLoading = false }

        let client = APIClient(session: session)
        do {
            async let eventsResult = client.listEvents(competitionId: competitionId)
            async let participantsResult = client.listParticipants(competitionId: competitionId)
            events = try await eventsResult.sorted { $0.orderIndex < $1.orderIndex }
            participants = try await participantsResult.filter(\.isAttending)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

#Preview {
    NavigationStack {
        JudgeHomeView(onSignOut: {})
            .environmentObject(ServerSession())
    }
}
