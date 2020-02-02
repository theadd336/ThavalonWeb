function onPageLoad(lobbyIds, playerNames) {
	let count = 0;
	for (let lobbyId of lobbyIds) {
		addNewLobby(lobbyId);
		addPlayers(playerNames[count], lobbyId);
		count++;
	}
}

function addPlayers(playerNames, lobbyId) {
	if (playerNames == null || lobbyId == null) { return; }
	const playerList = document.getElementById(lobbyId + "playerList");

	for (const playerName of playerNames) {
		let listForPlayerName = document.createElement("LI");
		listForPlayerName.classList.add("list-group-item");
		listForPlayerName.innerHTML = playerName;
		playerList.insertBefore(listForPlayerName, playerList.childNodes[0]);
	}
	return;
}

function addNewLobby(lobbyId) {
	if (document.getElementById(lobbyId + "playerList") !== null) {
		return;
	}
	// Get the div containing all lobbies.
	const allLobbies = document.getElementById("allLobbies");
	// Create the new div that will contain the new lobby. This is where the header is.
	const listDiv = createListDiv(lobbyId);
	// Append the button to join the game and its related forms.
	listDiv.appendChild(createNewPlayerList(lobbyId));
	// Append the new lobby to allLobies
	allLobbies.appendChild(listDiv);
	return;
}

function createListDiv(lobbyId) {
	// Create a new div with the appropriate classes and add a lobby header.
	const divNode = document.createElement("DIV");
	divNode.classList.add("list-group-item", "borderless", "div-lobby");
	divNode.innerHTML = "<h3>Lobby " + lobbyId + "</h3>";
	return divNode;
}

function createNewPlayerList(lobbyId) {
	// Create a new ordered list. This is where the player names will go.
	const listNode = document.createElement("OL");
	listNode.classList.add("list-group");
	listNode.id = lobbyId + "playerList";
	// At the bottom, append the list entry for the submit button and forms.
	listNode.appendChild(createListForForm(lobbyId));
	return listNode;
}

function createListForForm(lobbyId) {
	// Create the list entry for the submit button and the form.
	const listForFormNode = document.createElement("LI");
	listForFormNode.classList.add("list-group-item");
	// Append the form to the list entry and return it.
	listForFormNode.appendChild(setupForm(lobbyId));
	return listForFormNode;
}


function setupForm(lobbyId) {
	// Set up the
	const formNode = document.createElement("FORM");
	const formDivNode = document.createElement("DIV");
	formDivNode.classList.add("input-group");
	formDivNode.setAttribute("style", "width:250px;");
	formDivNode.appendChild(createOuterButton());
	formDivNode.appendChild(createNameForm(lobbyId));
	formDivNode.appendChild(createSpan(lobbyId));
	formNode.appendChild(formDivNode);
	return formNode;
}

function createOuterButton() {
	const outerButtonNode = document.createElement("BUTTON");
	outerButtonNode.setAttribute("type", "button");
	outerButtonNode.classList.add("btn", "btn-primary", "btnToggle");
	outerButtonNode.innerHTML = "Join This Lobby";
	return outerButtonNode;
}

function createNameForm(lobbyId) {
	const inputNode = document.createElement("INPUT");
	inputNode.setAttribute("type", "text");
	inputNode.classList.add("form-control", "with-border", "toggleMe");
	inputNode.id = lobbyId + "txtUserName";
	inputNode.setAttribute("style", "display:none;");
	inputNode.setAttribute("placeholder", "Enter display name");
	inputNode.setAttribute("onkeypress", "formEnter(e, this)");
	return inputNode;

}

function createSpan(lobbyId) {
	const spanNode = document.createElement("SPAN");
	spanNode.classList.add("input-group-btn");
	spanNode.appendChild(createSubmitButton(lobbyId));
	return spanNode;
}

function createSubmitButton(lobbyId) {
	const submitButtonNode = document.createElement("BUTTON");
	submitButtonNode.setAttribute("type", "button");
	submitButtonNode.classList.add("btn", "btn-primary", "toggleMe");
	submitButtonNode.setAttribute("style", "display:none");
	submitButtonNode.id = lobbyId;
	submitButtonNode.setAttribute("onclick", "joinGame(this.id)");
	submitButtonNode.setAttribute("href", "#");
	submitButtonNode.innerHTML = "Join!";
	return submitButtonNode;
}

$(function() {
    $(document).on("click",".btnToggle", function() {
        $(this).parent().find(".toggleMe").toggle();
        $(this).hide();
    })
});

function formEnter(e, form) {
    if (e.keyCode === 13) {  // enter, return
        const lobbyId = form.id.substring(0, form.length - "txtUserName".length);
        joinGame(lobbyId);
    }
}

function csrfSafeMethod(method) {
    // these HTTP methods do not require CSRF protection
    return (/^(GET|HEAD|OPTIONS|TRACE)$/.test(method));
}

function getCookie(name) {
	let cookieValue = null;
	if (document.cookie && document.cookie !== '') {
		let cookies = document.cookie.split(';');
			for (let i = 0; i < cookies.length; i++) {
				let cookie = cookies[i].trim();
				// Does this cookie string begin with the name we want?
				if (cookie.substring(0, name.length + 1) === (name + '=')) {
					cookieValue = decodeURIComponent(cookie.substring(name.length + 1));
					break;
				}
			}
		}
	return cookieValue;
}