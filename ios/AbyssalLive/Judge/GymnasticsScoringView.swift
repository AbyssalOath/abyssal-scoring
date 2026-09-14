import SwiftUI

/// Submits a gymnastics panel score via `POST /scores/gymnastics-panel`
/// (docs/LIVE_API.md). Sends one `JudgeScoreEntry` with `judgeId: nil`
/// - the server fills it in from the authenticated device, so a
/// single judge scoring from their own iPad never has to know their
/// own judge id.
struct GymnasticsScoringView: View {
    @EnvironmentObject private var session: ServerSession
    @Environment(\.dismiss) private var dismiss

    let event: MeetEvent
    let participant: Participant

    @State private var apparatus: Apparatus = .balanceBeam
    @State private var difficultyScore = "0.0"
    @State private var neutralDeductions = "0.0"
    @State private var deductions: [Deduction] = []
    @State private var newReason = ""
    @State private var newValue = ""
    @State private var isSubmitting = false
    @State private var errorMessage: String?
    @State private var submittedResult: EventResult?

    var body: some View {
        Form {
            Section(participant.fullName) {
                Picker("Apparatus", selection: $apparatus) {
                    ForEach(Apparatus.allCases) { item in
                        Text(item.label).tag(item)
                    }
                }
                LabeledContent("Difficulty (D)") {
                    TextField("0.0", text: $difficultyScore)
                        .keyboardType(.decimalPad)
                        .multilineTextAlignment(.trailing)
                }
                LabeledContent("Neutral deductions") {
                    TextField("0.0", text: $neutralDeductions)
                        .keyboardType(.decimalPad)
                        .multilineTextAlignment(.trailing)
                }
            }

            Section("Deductions") {
                ForEach(Array(deductions.enumerated()), id: \.offset) { index, deduction in
                    HStack {
                        Text(deduction.reason)
                        Spacer()
                        Text(String(format: "%.2f", deduction.value))
                            .foregroundStyle(.secondary)
                    }
                    .swipeActions {
                        Button("Delete", role: .destructive) {
                            deductions.remove(at: index)
                        }
                    }
                }

                HStack {
                    TextField("Reason", text: $newReason)
                    TextField("Value", text: $newValue)
                        .keyboardType(.decimalPad)
                        .frame(width: 70)
                        .multilineTextAlignment(.trailing)
                    Button("Add") {
                        addDeduction()
                    }
                    .disabled(newReason.isEmpty || Double(newValue) == nil)
                }
            }

            if let errorMessage {
                Section {
                    Text(errorMessage).foregroundStyle(.red)
                }
            }

            if let result = submittedResult {
                Section("Submitted") {
                    Text(String(format: "Final score: %.3f", result.finalValue ?? 0))
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
                            Text("Submit score").bold()
                        }
                        Spacer()
                    }
                }
                .disabled(isSubmitting || submittedResult != nil)
            }
        }
        .navigationTitle(apparatus.label)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Close") { dismiss() }
            }
        }
    }

    private func addDeduction() {
        guard let value = Double(newValue), !newReason.isEmpty else { return }
        deductions.append(Deduction(reason: newReason, value: value))
        newReason = ""
        newValue = ""
    }

    private func submit() async {
        errorMessage = nil
        guard let difficulty = Double(difficultyScore), let neutral = Double(neutralDeductions) else {
            errorMessage = "Difficulty and neutral deductions must be numbers."
            return
        }
        isSubmitting = true
        defer { isSubmitting = false }

        let body = GymnasticsScoreRequest(
            eventId: event.id,
            participantId: participant.id,
            apparatus: apparatus,
            difficultyScore: difficulty,
            judgeScores: [JudgeScoreEntry(judgeId: nil, deductions: deductions)],
            neutralDeductions: neutral
        )

        do {
            submittedResult = try await APIClient(session: session).submitGymnasticsScore(body)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

#Preview {
    NavigationStack {
        GymnasticsScoringView(
            event: MeetEvent(id: UUID(), competitionId: UUID(), sessionId: nil, name: "Beam", orderIndex: 1, status: "scheduled"),
            participant: Participant(id: UUID(), competitionId: UUID(), firstName: "Jane", lastName: "Doe", teamId: nil, bibNumber: nil, levelOrCategory: nil, isAttending: true)
        )
        .environmentObject(ServerSession())
    }
}
