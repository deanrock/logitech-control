//
//  AppDelegate.swift
//  LogitechControl
//
//  Created by dean on 03/04/2022.
//

import Foundation
import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var webSocketHandler: WebSocketHandler?
    private var statusItem: NSStatusItem!
    
    private var host: String = "localhost:8000"
    
    func eventToAction(event: NSEvent) -> String? {
        switch event.data1 {
        case 461312:
            return "mute"
        case 2560:
            return "volume_up"
        case 68096:
            return "volume_down"
        default:
            return nil
        }
    }
    
    func keyDown(event: NSEvent) {
        if (event.type.rawValue == 14 && event.subtype.rawValue == 8) {
            if let action = self.eventToAction(event: event) {
                let message = Message(action: action)
                do {
                    let data = try JSONEncoder().encode(message)
                    self.webSocketHandler?.send(message: String(data: data, encoding: .utf8)!)
                } catch {}
            }
        }
    }
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSEvent.addGlobalMonitorForEvents(matching: .any, handler: self.keyDown)
        self.webSocketHandler = WebSocketHandler(url: "ws://\(self.host)/ws")
        
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        // 3
        if let button = statusItem.button {
            button.image = NSImage(systemSymbolName: "1.circle", accessibilityDescription: "1")
        }
        
        let menu = NSMenu()
        
        // 2
        let one = NSMenuItem(title: "Open page", action: #selector(openPage) , keyEquivalent: "1")
        menu.addItem(one)
        
        let two = NSMenuItem(title: "Two", action: #selector(didTapTwo) , keyEquivalent: "2")
        menu.addItem(two)
        
        let three = NSMenuItem(title: "Three", action: #selector(didTapThree) , keyEquivalent: "3")
        menu.addItem(three)
        
        menu.addItem(NSMenuItem.separator())
        
        menu.addItem(NSMenuItem(title: "Quit", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))
        
        // 3
        statusItem.menu = menu
    }
    
    private func changeStatusBarButton(number: Int) {
        if let button = statusItem.button {
            button.image = NSImage(systemSymbolName: "\(number).circle", accessibilityDescription: number.description)
        }
    }
    
    @objc func openPage() {
        if let url = URL(string: "http://\(self.host)") {
            NSWorkspace.shared.open(url)
        }
    }
    
    @objc func didTapTwo() {
        changeStatusBarButton(number: 2)
    }
    
    @objc func didTapThree() {
        changeStatusBarButton(number: 3)
    }
}
