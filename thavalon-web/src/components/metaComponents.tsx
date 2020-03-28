import React from "react";

export class DocMeta extends React.Component {
    componentDidMount(): void {
        document.title = "Thavalon";
    }
    render(): JSX.Element {
        return (<></>);
    }
}