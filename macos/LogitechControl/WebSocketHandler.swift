//
//  WebSocketHandler.swift
//  LogitechControl
//
//  Created by dean on 03/04/2022.
//

import Foundation

class WebSocketHandler: NSObject, URLSessionWebSocketDelegate {
    let url: String
    var socket: URLSessionWebSocketTask!
    
    init(url: String) {
        self.url = url
        super.init()
        
        Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { _ in
            self.ping()
        }
        
        self.connect()
    }
    
    @objc func connect() {
        let session = URLSession(configuration: .default, delegate: self, delegateQueue: OperationQueue())
        
        self.socket = session.webSocketTask(with: URL(string: url)!)
        self.socket.resume()
        
        self.listen()
    }
    
    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didOpenWithProtocol protocol: String?) {
    }
    
    // No idea when this is triggered. Doesn't happen if server is inacessible, or if it's stopped during the app run time.
    func urlSession(_ session: URLSession, webSocketTask: URLSessionWebSocketTask, didCloseWith closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        self.triggerReconnect()
    }
    
    func triggerReconnect() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
            self.connect()
        }
    }
    
    func listen() {
        self.socket.receive { result in
            switch result {
            case .failure(_):
                self.triggerReconnect()
                return
            case .success(let message):
                switch message {
                case .string(let text):
                    print("Received text message: \(text)")
                case .data(let data):
                    print("Received binary message: \(data)")
                @unknown default:
                    fatalError()
                }
            }
            
            self.listen()
        }
    }
    
    public func send(message: String) {
        self.socket.send(URLSessionWebSocketTask.Message.string(message)) { error in
            if let error = error {
                print("\(error)")
            }
        }
    }
    
    @objc func ping() {
        self.socket.sendPing { (error) in
            if let error = error {
                print("Ping failed: \(error)")
            }
        }
    }
}
