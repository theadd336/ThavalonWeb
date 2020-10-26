import { read } from 'fs';
import React, { ChangeEvent } from 'react';

export function Account() {
    const profileImage = React.useRef<HTMLImageElement>(null);
    function handleImageUpload(event: ChangeEvent<HTMLInputElement>): void {
        if (profileImage.current === null) {
            console.log("Unable to access image element, cannot update profile picture");
        }

        // get the chosen file. There should be only 1 because multiple choices disabled.
        const file = event.target.files;
        if (file !== null && file.length === 1) {
            console.log(file.item(0));
            const reader = new FileReader();
            let image: HTMLImageElement = profileImage.current;
            
            reader.onload = (e) => {
                image?.src = e.target.result;
            }
        }
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