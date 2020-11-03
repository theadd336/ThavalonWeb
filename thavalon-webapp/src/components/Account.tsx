import React, { ChangeEvent } from 'react';

export function Account(): JSX.Element {
    const profileImage = React.useRef<HTMLImageElement>(null);
    function handleImageUpload(event: ChangeEvent<HTMLInputElement>): void {
        // get the chosen file. There should be only 1 because multiple choices disabled.
        const file = event.target.files;
        if (file === null || file.length !== 1) {
            return;
        }
        const fileItem = file.item(0);
        if (fileItem === null) {
            return;
        }

        let imageElement: HTMLImageElement | null = profileImage.current;
        const reader = new FileReader();
        reader.onload = (event: ProgressEvent<FileReader>) => {
            // verify not accessing null elements on anything
            if (imageElement === null || event.target === null || event.target.result === null) {
                return;
            }
            imageElement.src = event.target.result as string;
        }
        reader.readAsDataURL(fileItem);
    }

    console.log("Rendering account!");
    return (
        <div>
            <h1>Accout Page!</h1>
            <input
                type="file"
                accept="image/*"
                multiple={false}
                onChange={handleImageUpload} />
            <img alt="Profile" ref={profileImage} />
        </div>
    );
}