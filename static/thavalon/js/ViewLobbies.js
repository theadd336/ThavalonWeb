function csrfSafeMethod(method) {
    // these HTTP methods do not require CSRF protection
    return (/^(GET|HEAD|OPTIONS|TRACE)$/.test(method));
}

function addNewLobby(lobbyId) {
	if (document.getElementById(lobbyId + "playerList") !== null) {
		return;
	}

	let allLobbies = document.getElementById("allLobbies");
	let newPlayerList = createNewList(lobbyId);
	allLobbies.appendChild(newPlayerList);
	return;
}

function createNewList(lobbyId) {
	let listNode = document.createElement("LI");
	listNode.classList.add("list-group-item");
	listNode.id = lobbyId + "playerList";
	listNode.innerHTML = "<h3>Lobby " + lobbyId + "</h3>";
	listNode.appendChild(createUnorderedList(lobbyId));
	return listNode;
}

function createUnorderedList(lobbyId) {
	let unorderedListNode = document.createElement("UL");
	unorderedListNode.classList.add("list-group");
	unorderedListNode.id = lobbyId + "unorderedList";
	unorderedListNode.appendChild(addJoinButton(lobbyId));
	return unorderedListNode;
}

function addJoinButton(lobbyId) {
	let joinButtonNode = document.createElement("LI");
	joinButtonNode.classList.add("list-group-item");
	joinButtonNode.id = lobbyId + "joinButton";
	joinButtonNode.appendChild(formatJoinButtonLink(lobbyId))
	return joinButtonNode;
}

function formatJoinButtonLink(lobbyId) {
	let linkNode = document.createElement("A");
	linkNode.setAttribute("onclick", "");
	linkNode.setAttribute("href","");
	linkNode.classList.add("btn-join");
	linkNode.classList.add("text-success")
	linkNode.innerHTML = "Join";
	linkNode.id = lobbyId + "link";
	return linkNode;
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