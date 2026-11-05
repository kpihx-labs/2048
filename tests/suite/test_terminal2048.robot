*** Settings ***
Documentation    Visual smoke test for the native responsive 2048 binary
Resource         kvm.resource
Library          Process


*** Test Cases ***
Terminal 2048 Launches And Renders
    [Documentation]    Verify the installed native binary launches in a foot terminal on a real GNOME desktop session
    [Tags]    smoke    yarf:certification_status: blocker
    Start Process    /usr/bin/foot    2048    --layout    auto    alias=terminal-2048
    Sleep    3s
    Log Screenshot
    [Teardown]    Terminate Process    terminal-2048
