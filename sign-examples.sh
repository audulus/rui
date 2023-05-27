#!/bin/zsh

# This re-signs the examples so they can be profiled with Instruments on macOS.
# See https://developer.apple.com/forums/thread/681687

# Make sure to set the shell to zsh, not bash
#
# For Instruments, re-sign binary with get-task-allow entitlement
codesign -s - -v -f --entitlements =(echo -n '<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd"\>
<plist version="1.0">
    <dict>
        <key>com.apple.security.get-task-allow</key>
        <true/>
    </dict>
</plist>') target/debug/examples/*
