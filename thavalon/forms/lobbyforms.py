from django import forms


class JoinGameForm(forms.Form):
    join_game = forms.HiddenInput()