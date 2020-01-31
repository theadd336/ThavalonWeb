function populateGameState(gamestate) {
    if (gamestate === null || gamestate === undefined) { return; }
    populateRoleBlurb(gamestate.roleInformation);
    populateRoleInformation(gamestate.roleInformation.information);
    populatePlayerOrder(gamestate.proposalOrder);
    alert(gamestate.proposalOrder);
    alert(gamestate.proposerIndex);
    switch (gamestate.currentPhase) {
        case 1:
            populateProposalTab(gamestate.isProposing,
            gamestate.proposerIndex,
            gamestate.proposalOrder,
            gamestate.proposalNumber,
            gamestate.maxNumProposals,
            gamestate.missionSizes[gamestate.missionNum + 1],
            gamestate.currentProposal);
            break;
    }
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

function populatePlayerOrder(playerOrder) {
    const playerOrderNode = document.getElementById("playerOrderLocation");
    let count = 0;
    for (let playerName of playerOrder) {
        let playerListEntry = document.createElement("LI");
        playerListEntry.textContent = playerName;
        playerListEntry.id = count;
        playerOrderNode.appendChild(playerListEntry);
        count++;
    }
}

function populateProposalTab(isProposing, proposerIndex, proposalOrder, proposalNumber,
maxNumProposals, numOnMission, currentProposal) {
    writeProposalHeader(isProposing, proposalOrder[proposerIndex], proposalNumber, maxNumProposals);
    if (isProposing) {
        writeProposalBodyProposing(proposalOrder, numOnMission);
    } else {
        writeProposalBodyOther(proposerName, currentProposal);
    }
}

function writeProposalHeader(isProposing, proposerName, proposalNumber, maxNumProposals, numOnMission) {
    // Set the tab name to "Proposals"
    const tabHeader = document.getElementById("nav-profile-tab");
    tabHeader.textContent = "Proposals";

    // Write the header section. This includes proposal number, whether it's force, and who is proposing.
    const proposalHeaderSection = document.getElementById("proposalVoteHeader");
    proposalHeaderSection.textContent = `Proposal ${proposalNumber}/${maxNumProposals}`;
    if (proposalNumber === maxNumProposals) {
        const forceIndicatorNode = document.createElement("SPAN");
        forceIndicatorNode.classList.add("text-danger");
        forceIndicatorNode.textContent = "[FORCE]";
        proposalHeaderSection.appendChild(forceIndicatorNode);
    }

    // blank line for formatting
    proposalHeaderSection.appendChild(document.createElement("BR"));

    // Write the sentence on who is proposing;
    let proposerSentence = "";
    if (isProposing) {
        proposerSentence += "You are ";
    } else {
        proposerSentence += proposerName + " is ";
    }
    proposerSentence += `proposing a ${numOnMission} person mission.`;
    const sentenceTextNode = document.createTextNode(proposerSentence);
    proposalHeaderSection.appendChild(sentenceTextNode);
}

function writeProposalBodyOther(proposerName, currentProposal) {
    const proposalBodySection = document.getElementById("proposalVoteContent");
    if (currentProposal == null || currentProposal.length === 0) {
        proposalBodySection.textContent = `Please wait while ${proposerName} proposes a mission`;
        return;
    }
    proposalBodySection.textContent = `${proposerName} has proposed:`
    const listNode = document.createElement("UL");
    for (const playerName of currentProposal) {
        const listEntry = document.createElement("LI");
        listEntry.textContent = playerName;
        listNode.appendChild(listEntry);
    }
    proposalBodySection.appendChild(listNode);
}

function writeProposalBodyProposing(playerOrder, test) { return; }