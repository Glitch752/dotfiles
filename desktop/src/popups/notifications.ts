import { listen } from "@tauri-apps/api/event";
import { Notification } from "@bindings/Notification";
import { closePopup, openPopup, PopupType } from "./popups";
import { createIconImage } from "../components/iconImage";

// TODO: Notification center so notifications don't just permanently disappear

const notifications: Map<number, {
    data: Notification,
    element: HTMLElement
}> = new Map();

const NOTIFICATION_TIMEOUT = 8000;

let notificationAnchor = document.createElement("div");
notificationAnchor.style.position = "absolute";
notificationAnchor.style.top = "0";
notificationAnchor.style.right = "0";
document.body.appendChild(notificationAnchor);

let notificationsElement = document.createElement("div");
notificationsElement.className = "notifications-container";

function createNotification(notification: Notification): HTMLElement {
    const notificationContainerElement = document.createElement("div");
    notificationContainerElement.className = `notification-container ${notification.urgency.toLowerCase()}`;

    const notificationElement = document.createElement("div");
    notificationElement.className = "notification";
    notificationContainerElement.appendChild(notificationElement);

    const applicationContainer = document.createElement("div");
    applicationContainer.className = "application";

    if(notification.application_name) {
        const applicationName = document.createElement("div");
        applicationName.className = "application-name";
        applicationName.textContent = notification.application_name;
        applicationContainer.appendChild(applicationName);
    }
    if(notification.application_icon) {
        const icon = createIconImage(notification.application_icon);
        icon.className = "icon";
        applicationContainer.appendChild(icon);
    }

    const title = document.createElement("span");
    title.className = "title";
    title.textContent = notification.title;

    const body = document.createElement("div");
    body.className = "message";
    body.textContent = notification.body;
    
    notificationElement.append(applicationContainer, title, body);

    return notificationContainerElement;
}

function removeNotification(id: number) {
    const notification = notifications.get(id);
    if(!notification) return;

    notifications.delete(id);

    function animateOut() {
        if(!notification) return;
        
        notification.element.animate([
            { opacity: 1, translate: "0 0", maxHeight: notification.element.scrollHeight + "px" },
            { opacity: 0, translate: "30px 0", maxHeight: "0" }
        ], {
            duration: 200,
            easing: "cubic-bezier(0.25, 0.1, 0.25, 1.2)",
        });
        setTimeout(() => {
            notificationsElement.removeChild(notification.element);
        }, 200);
    }

    const notificationsOpen = notifications.size > 0;
    if(!notificationsOpen) {
        closePopup(PopupType.Notifications, "notifications");
        setTimeout(() => {
            animateOut();
        }, 600);
    } else {
        animateOut();
    }
}

export function initNotifications() {
    listen<Notification>("notification_added", (event) => {
        if(!notificationAnchor) return;

        const notification = event.payload;

        const notificationsOpen = notifications.size > 0;
        
        const element = createNotification(notification);
        notifications.set(notification.id, {
            data: notification,
            element
        });

        let timeout = setTimeout(() => {
            if(!notifications.has(notification.id)) return;
            removeNotification(notification.id);
        }, NOTIFICATION_TIMEOUT);

        element.onclick = () => {
            clearTimeout(timeout);
            removeNotification(notification.id);
        }

        notificationsElement.append(element);

        if(!notificationsOpen) {
            openPopup(PopupType.Notifications, "notifications", notificationsElement, notificationAnchor);
        }

        element.animate([
            { opacity: 0, translate: "30px 0", maxHeight: notificationsOpen ? "0" : element.clientHeight + "px" },
            { opacity: 1, translate: "0 0", maxHeight: element.clientHeight + "px" }
        ], {
            duration: 200,
            easing: "cubic-bezier(0.25, 0.1, 0.25, 1.2)",
        });
    });
    listen<number>("notification_removed", (event) => {
        const id = event.payload;
        removeNotification(id);
    });
}