function populateGameState(gamestate) {
    if (gamestate === null || gamestate === undefined) { return; }
    populateRoleBlurb(gamestate.roleInformation);
    populateRoleInformation(gamestate.roleInformation.information);
    populatePlayerOrder(gamestate.proposalOrder);
    switch (gamestate.currentPhase) {
        case 0:
            populateProposalTab(
                gamestate.isProposing,
                gamestate.proposerIndex,
                gamestate.proposalOrder,
                gamestate.proposalNum,
                gamestate.maxNumProposals,
                gamestate.proposalSize,
                gamestate.currentProposal);
            break;
        case 1:

            onMoveToVote(
                {"playerList": gamestate.currentProposal}
                );
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
        spans[1].classList.add("text-success");
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
    writeProposalHeader(isProposing, proposalOrder[proposerIndex], proposalNumber, maxNumProposals, numOnMission);
    if (isProposing) {
        writeProposalBodyProposing(proposalOrder, numOnMission);
    } else {
        writeProposalBodyOther(proposalOrder[proposerIndex], currentProposal);
    }
}

function writeProposalHeader(isProposing, proposerName, proposalNumber, maxNumProposals, numOnMission) {
    // Set the tab name to "Proposals"
    const tabHeader = document.getElementById("nav-profile-tab");
    tabHeader.textContent = "Proposals";

    // Write the header section. This includes proposal number, whether it's force, and who is proposing.
    const proposalHeaderSection = document.getElementById("proposalVoteHeader");
    // Clear out all old content before writing new information.
    proposalHeaderSection.innerHTML = "";
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
        proposalBodySection.textContent = `Please wait while ${proposerName} proposes a mission.`;
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

function writeProposalBodyProposing(playerOrder, numOnMission) {
    
    // TODO: handle the current proposal if player disconnects and reconnects during proposal.
    // Get the template for the proposer selection list and its location.
    const proposerSelectionListTemplate = document.getElementById("proposerSelectionListTemplate");
    const proposerSelectionListLocation = document.getElementById("proposalVoteContent");
    // Clone the template and add the options.
    const proposerSelectionList = proposerSelectionListTemplate.content.cloneNode(true);
    const selectNode = proposerSelectionList.querySelector("select");
    selectNode.setAttribute("data-max-options", numOnMission);
    selectNode.id = "proposedPlayerList";
    for (const playerName of playerOrder) {
        const optionNode = document.createElement("OPTION");
        optionNode.setAttribute("value", playerName);
        optionNode.textContent = playerName;
        selectNode.appendChild(optionNode);
    }
    // Clear old values from the proposal tab.
    proposerSelectionListLocation.innerHTML = ""
    proposerSelectionListLocation.appendChild(proposerSelectionList);
    $('#proposedPlayerList').selectpicker('render');
    return;
}

function onPropose(proposalInfo) {
    writeProposalBodyOther(proposalInfo.proposerName, proposalInfo.proposedPlayerList);
}

function onMoveToVote(proposalInfo) {
    writeVoteHeader();
    writeVoteBody(proposalInfo.playerList)
}

function writeVoteHeader() {
    // Set the tab name to "Voting"
    const tabHeader = document.getElementById("nav-profile-tab");
    tabHeader.textContent = "Voting";
}

function writeVoteBody(playerList) {
    const voteBodySection = document.getElementById("proposalVoteContent");
    // Clear the old values before writing voting information.
    voteBodySection.innerHTML = "";
    voteBodySection.textContent = "Voting on:"
    const listNode = document.createElement("UL");
    for (const playerName of playerList) {
        const listEntry = document.createElement("LI");
        listEntry.textContent = playerName;
        listNode.appendChild(listEntry);
    }
    voteBodySection.appendChild(listNode);

    const votingButtonsTemplate = document.getElementById("voteButtonsTemplate");
    const votingButtons = votingButtonsTemplate.content.cloneNode(true);
    voteBodySection.appendChild(votingButtons)
}

function newProposal(message) {
    populateProposalTab(
        message.isProposing,
        message.proposerIndex,
        message.proposalOrder,
        message.proposalNum,
        message.maxNumProposals,
        message.proposalSize,
        message.currentProposal);
    if (message.priorVoteInfo != null) {
        writePriorProposalVoteResults(message.priorVoteInfo);
    }
}

function onMissionStart(message) {
    if (message.isOnMission) {
        populateMissionTabOnMission();
    } else {
        populateMissionTabNotOnMission(message.playerList);
    }
    const voteBodySection = document.getElementById("proposalVoteContent");
    voteBodySection.innerHTML = "";
    voteBodySection.textContent = "Mission is going.";
    console.log(message.priorVoteInfo)
    if (message.priorVoteInfo != null) {
        writePriorProposalVoteResults(message.priorVoteInfo);
    }
}

function populateMissionTabOnMission() {
    const missionBodyLocation = document.getElementById("nav-about");
    const missionBodyTemplate = document.getElementById("onMissionTemplate");
    const missionBodyOnMission = missionBodyTemplate.content.cloneNode(true);
    missionBodyLocation.innerHTML = "";
    missionBodyLocation.appendChild(missionBodyOnMission);
}

function populateMissionTabNotOnMission(playersOnMission) {
    const missionBodyLocation = document.getElementById("nav-about");
    const missionBodyTemplate = document.getElementById("notOnMissionTemplate");
    const missionBodyNotOnMission = missionBodyTemplate.content.cloneNode(true);
    missionBodyLocation.innerHTML = "";
    const preNode = missionBodyNotOnMission.querySelector("pre");
    let missionSentence = "Please wait while " + playersOnMission.join(",") + " go on a mission.";
    if (playersOnMission.includes("Meg")) {
        missionSentence += " Don't fail it Meg!";
    }
    preNode.textContent = missionSentence;
    missionBodyLocation.appendChild(missionBodyNotOnMission);
}

function onMissionResults(message) {
    const missionResult = message.missionResult;
    const priorMissionNum = message.priorMissionNum;
    let missionResultTemplate = null
    if (missionResult === 0) {
        missionResultTemplate = document.getElementById("missionPassedTemplate");
    } else {
        missionResultTemplate = document.getElementById("missionFailedTemplate");
    }

    const missionIndicatorLocation = document.getElementById("m" + priorMissionNum + "Indicator");
    missionIndicatorLocation.innerHTML = "";
    const missionResultNode = missionResultTemplate.content.cloneNode(true);
    missionIndicatorLocation.appendChild(missionResultNode);
    const missionBodyLocation = document.getElementById("nav-about");
    missionBodyLocation.innerHTML = "";
    missionBodyLocation.textContent = message.playedCards;
}

function voteStillInProgress(message) {
    // Get the location of the vote information and the position of the last element (the buttons).
    const voteBodySection = document.getElementById("proposalVoteContent");
    // Remove the buttons and put a message in their place to inform the user voting is complete.
    voteBodySection.innerHTML = "";
    // Format the sentence based on the vote boolean.
    let alreadyVotedSentence = "You have ";
    if (message.submittedVote) {
        alreadyVotedSentence += "upvoted. ";
    } else {
        alreadyVotedSentence += "downvoted. ";
    }
    alreadyVotedSentence += "Please wait while others finish voting."
    // Create the text node and add it to the voting section.
    const alreadyVotedTextNode = document.createTextNode(alreadyVotedSentence);
    voteBodySection.appendChild(alreadyVotedTextNode);
}

function writePriorProposalVoteResults(priorVoteInfo) {
    const priorVoteInfoLocation = document.getElementById("nav-home");
    priorVoteInfoLocation.innerHTML = "";
    priorVoteInfoLocation.textContent = "Prior proposal votes:";
    if (priorVoteInfo.wasObscured) {
        priorVoteInfoLocation.appendChild(document.createTextNode("Someone has obscured the votes."));
    }
    const voteListNode = document.createElement("UL");
    let vote = "";
    for (const playerName in priorVoteInfo) {
        const voteListEntry = document.createElement("LI");
        voteListEntry.textContent = playerName + ": "
        if (priorVoteInfo.playerName === True) {
            vote = "Upvoted";
        } else if (priorVoteInfo.playerName === False) {
            vote = "Downvoted";
        } else {
            vote = priorVoteInfo.playerName;
        }
        voteListEntry.textContent += vote;
        voteListNode.appendChild(voteListEntry);
    }
    priorVoteInfoLocation.appendChild(voteListNode);
}

function missionStillInProgress(message) {
    const missionBodyLocation = document.getElementById("nav-about");
    missionBodyLocation.innerHTML = "";
    const cardPlayed = message.cardPlayed;
    let missionInProgressSentence = "You have played a " + cardPlayed + ". ";
    if (cardPlayed === "SUCCESS") {
        missionInProgressSentence += "Good job!";
    } else if (cardPlayed === "FAIL") {
        missionInProgressSentence += "Why did you have to fail :(.";
    } else {
        "I see a bus in your future.";
    }
    missionBodyLocation.textContent = missionInProgressSentence;
}