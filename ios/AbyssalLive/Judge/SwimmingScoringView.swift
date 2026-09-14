import SwiftUI

/// Submits a swimming/track time via `POST /scores/swimming`
/// (docs/LIVE_API.md).
struct SwimmingScoringView: View {
    @EnvironmentObject private var session: ServerSession
    @Environment(\.dismiss) private var dismiss

    let event: MeetEvent
    let participant: Participant

    @State private var elapsedSeconds = "0.0"
    @State private var timePenaltySeconds = "0.0"
    @State private var isSubmitting = false
    @State private var errorMessage: String?
    @State private var submittedResult: EventResult?

    var body: some View {
        Form {
            Section(participant.fullName) {
                LabeledContent("Elapsed time (s)") {
                    TextField("0.0", text: $elapsedSeconds)
                        .keyboardType(.decimalPad)
                        .multilineTextAlignment(.trailing)
                }
                LabeledContent("Time penalty (s)") {
                    TextField("0.0", text: $timePenaltySeconds)
                        .keyboardType(.decimalPad)
                        .multilineTextAlignment(.trailing)
                }
            }

            if let errorMessage {
                Section {
                    Text(errorMessage).foregroundStyle(.red)
                }
            }

            if let result = submittedResult {
                Section("Submitted") {
                    Text(String(format: "Final time: %.2fs", result.finalValue ?? 0))
                        .font(.headline)
                }
            }

            Section {
                Button {
                    Task { await submit() }
                } label: {
                    HStack {
                        Spacer()
                        if isSubmitting {
                            ProgressView()
                        } else {
                            Text("Submit time").bold()
                        }
                        Spacer()
                    }
                }
                .disabled(isSubmitting || submittedResult != nil)
            }
        }
        .navigationTitle("Swimming / Track")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Close") { dismiss() }
            }
        }
    }

    private func submit() async {
        errorMessage = nil
        guard let elapsed = Double(elapsedSeconds), let penalty = Double(timePenaltySeconds) else {
            errorMessage = "Elapsed time and penalty must be numbers."
            return
        }
        isSubmitting = true
        defer { isSubmitting = false }

        let body = SwimmingScoreRequest(
            eventId: event.id,
            participantId: participant.id,
            elapsedSeconds: elapsed,
            timePenaltySeconds: penalty
        )

        do {
            submittedResult = try await APIClient(session: session).submitSwimmingScore(body)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

#Preview {
    NavigationStack {
        SwimmingScoringView(
            event: MeetEvent(id: UUID(), competitionId: UUID(), sessionId: nil, name: "50m Freestyle", orderIndex: 1, status: "scheduled"),
            participant: Participant(id: UUID(), competitionId: UUID(), firstName: "Jane", lastName: "Doe", teamId: nil, bibNumber: nil, levelOrCategory: nil, isAttending: true)
        )
        .environmentObject(ServerSession())
    }
}
