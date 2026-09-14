import SwiftUI

/// No pairing required - just a server address and a competition to
/// watch, using only public endpoints (`GET /competitions`,
/// `GET /scoreboard/:id`, `GET /ws/scoreboard/:id`). See
/// docs/LIVE_API.md.
struct SpectatorFlowView: View {
    @EnvironmentObject private var session: ServerSession
    let onExit: () -> Void

    @State private var host = ""
    @State private var port = "7878"
    @State private var competitions: [Competition] = []
    @State private var selectedCompetition: Competition?
    @State private var isLoading = false
    @State private var errorMessage: String?

    var body: some View {
        Group {
            if let competition = selectedCompetition {
                SpectatorScoreboardView(competition: competition) {
                    selectedCompetition = nil
                }
            } else {
                connectForm
            }
        }
    }

    private var connectForm: some View {
        Form {
            Section("Server") {
                TextField("IP address, e.g. 192.168.1.42", text: $host)
                    .keyboardType(.numbersAndPunctuation)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                TextField("Port", text: $port)
                    .keyboardType(.numberPad)
                Button {
                    Task { await connect() }
                } label: {
                    HStack {
                        Spacer()
                        if isLoading {
                            ProgressView()
                        } else {
                            Text("Connect")
                        }
                        Spacer()
                    }
                }
                .disabled(host.isEmpty || isLoading)
            }

            if let errorMessage {
                Section {
                    Text(errorMessage).foregroundStyle(.red)
                }
            }

            if !competitions.isEmpty {
                Section("Choose a competition") {
                    ForEach(competitions) { competition in
                        Button {
                            selectedCompetition = competition
                        } label: {
                            VStack(alignment: .leading) {
                                Text(competition.name)
                                Text(competition.date).font(.caption).foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("Watch Live")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Back", action: onExit)
            }
        }
        .onAppear {
            host = session.host
            port = String(session.port)
        }
    }

    private func connect() async {
        errorMessage = nil
        guard let portNumber = Int(port) else {
            errorMessage = "Port must be a number."
            return
        }
        isLoading = true
        defer { isLoading = false }

        session.saveConnection(host: host, port: portNumber)
        do {
            competitions = try await APIClient(session: session).listCompetitions()
            if competitions.isEmpty {
                errorMessage = "Connected, but no competitions exist on this server yet."
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

#Preview {
    NavigationStack {
        SpectatorFlowView(onExit: {})
            .environmentObject(ServerSession())
    }
}
