function populateGameState(gamestate) {
    if (gamestate === null || gamestate === undefined) { return; }
    populateRoleBlurb(gamestate.roleInformation);
}

function populateRoleBlurb(roleInformation) {
    const roleBlurbTemplate = document.getElementById("roleBlurbTemplate");
    const roleBlurbLocation = document.getElementById("roleBlurbLocation");
    const roleBlurb = roleBlurbTemplate.content.cloneNode(true);
    const span = roleBlurb[0];
    span[0].textContent = "You are " + roleInformation.role;

    const innerSpanNode = document.createElement("SPAN");
    let team = "";
    if (roleInformation.team === 1) {
        team = "[EVIL]";
        innerSpanNode.classList.add("text-danger")
    } else {
        team = "[GOOD]";
    }
    innerSpanNode[0].textContent = team;
    span.appendChild(innerSpanNode);
    roleBlurbLocation.appendChild(roleBlurb);
}
