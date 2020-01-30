function populateGameState(gamestate) {
    if (gamestate === null || gamestate === undefined) { return; }
    populateRoleBlurb(gamestate.roleInformation);
    populateRoleInformation(gamestate.roleInformation.information);
}

function populateRoleBlurb(roleInformation) {
    // Get the location of the role blurb template and the template itself.
    const roleBlurbTemplate = document.getElementById("roleBlurbTemplate");
    const roleBlurbLocation = document.getElementById("roleBlurbLocation");
    // Clone the template and find all the spans.
    const roleBlurb = roleBlurbTemplate.content.cloneNode(true);
    const spans = roleBlurb.querySelectorAll("span");
    // Add role information to the blurb.
    spans[0].textContent = "You are " + roleInformation.role;

    // Add team information to the blurb.
    let team = "";
    if (roleInformation.team === 1) {
        team = " [EVIL]";
        spans[1].classList.add("text-danger")
    } else {
        team = " [GOOD]";
    }
    // Add everything to the role location.
    spans[1].textContent = team;
    spans[0].appendChild(spans[1]);
    roleBlurbLocation.appendChild(roleBlurb);
}

function populateRoleInformation(information) {
    const roleInformationLocation = document.getElementById("roleInformationLocation");
    roleInformationLocation.textContent = "-------------------------\r\n";
    roleInformationLocation.textContent += information + "\r\n";
    roleInformationLocation.textContent += "-------------------------";
}
