import SwiftUI
import UIKit

/// Enter the server address + a pairing code generated on the
/// desktop's "Live Judging" tab (see docs/LIVE_API.md's `POST /pair`).
/// No QR scanning yet - a reasonable follow-up, but typing a 6-digit
/// code is a small enough ask to skip that complexity for now.
struct PairingView: View {
    @EnvironmentObject private var session: ServerSession

    @State private var host = ""
    @State private var port = "7878"
    @State private var code = ""
    @State private var deviceName = UIDevice.current.name
    @State private var isConnecting = false
    @State private var errorMessage: String?

    var body: some View {
        Form {
            Section {
                Text("Ask the coach for the server address and generate a pairing code for you on the desktop's \"Live Judging\" tab.")
                    .foregroundStyle(.secondary)
            }

            Section("Server") {
                TextField("IP address, e.g. 192.168.1.42", text: $host)
                    .keyboardType(.numbersAndPunctuation)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                TextField("Port", text: $port)
                    .keyboardType(.numberPad)
            }

            Section("Pairing code") {
                TextField("6-digit code", text: $code)
                    .keyboardType(.numberPad)
                TextField("Device name", text: $deviceName)
                    .autocorrectionDisabled()
            }

            if let errorMessage {
                Section {
                    Text(errorMessage).foregroundStyle(.red)
                }
            }

            Section {
                Button {
                    Task { await pair() }
                } label: {
                    HStack {
                        Spacer()
                        if isConnecting {
                            ProgressView()
                        } else {
                            Text("Pair").bold()
                        }
                        Spacer()
                    }
                }
                .disabled(host.isEmpty || code.isEmpty || isConnecting)
            }
        }
        .navigationTitle("Pair Device")
        .onAppear {
            host = session.host
            port = String(session.port)
        }
    }

    private func pair() async {
        errorMessage = nil
        guard let portNumber = Int(port) else {
            errorMessage = "Port must be a number."
            return
        }
        isConnecting = true
        defer { isConnecting = false }

        session.saveConnection(host: host, port: portNumber)
        let client = APIClient(session: session)
        do {
            let response = try await client.pair(code: code.trimmingCharacters(in: .whitespaces), deviceName: deviceName)
            session.completePairing(token: response.token, judge: response.judge)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

#Preview {
    NavigationStack {
        PairingView()
            .environmentObject(ServerSession())
    }
}
