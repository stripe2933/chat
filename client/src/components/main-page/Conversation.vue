<script setup>
import ConversationThumbnail from './ConversationThumbnail.vue';
import ProfileThumbnail from '../ProfileThumbnail.vue';
import { computed, nextTick, ref, watch } from 'vue';
import { onMounted } from 'vue';
import { onUnmounted } from 'vue';

const props = defineProps({
    conversation_id: {
        type: Number,
        required: true
    },
    self: {
        type: Object,
        required: true
    }
});

const conversation = ref(null);
let isDataLoaded = ref(false);

const messages = ref([]);
let currentMessage = ref('');

// Socket settings.
let socket = null;

function disconnect(){
    if (socket){
        console.log('Disconnecting...');
        socket.close();
        socket = null;
    }
}

function connect(){
    disconnect();

    const wsUri = `wss://localhost:8443/ws/`;
    socket = new WebSocket(wsUri);

    socket.onopen = () => {
        // socketStatus = true;
    };

    socket.onclose = () => {
        // socketStatus = false;
    };

    socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        if (data.JoinStatus){
            if (!data.JoinStatus.success){
                alert('Failed to join conversation.');
            }
        }
        else if (data.message){
            messages.value.push(data.message);

            // Should not directly call scrollMessageSectionToEnd() here, because the DOM is not updated yet.
            nextTick(() => {
                scrollMessageSectionToEnd();
            });
        }
        else{
            console.log('Websocket error');
        }
    };

}

onMounted(() => {
    connect();
});

onUnmounted(() => {
    disconnect();
});

watch(() => props.conversation_id, async (new_conversation_id, _) => {
    isDataLoaded.value = false;

    // Fetch conversation by given id.
    const conversationResponse = await fetch(`https://localhost:8443/api/conversation/${new_conversation_id}`, {mode: 'cors', credentials: 'include'});
    conversation.value = await conversationResponse.json();
    
    // Fetch previously sent messages.
    const messageResponse = await fetch(`https://localhost:8443/api/conversation/${new_conversation_id}/messages`, {mode: 'cors', credentials: 'include'});
    messages.value = await messageResponse.json();

    isDataLoaded.value = true;

    nextTick(() => {
        scrollMessageSectionToEnd();
    });

    // Connect socket to the new conversation.
    socket.send(`/join ${new_conversation_id}`)
}, { immediate: true /* Fetch messages when onMounted */ });

function chunkBy(arr, predicate){
    if (arr.length === 0){
        return [];
    }

    let chunked = [];
    let currentChunk = [arr[0]];

    for (let i = 1; i < arr.length; ++i){
        if (predicate(arr[i - 1], arr[i])) {
            currentChunk.push(arr[i]);
        }
        else{
            chunked.push(currentChunk);
            currentChunk = [arr[i]];
        }
    }

    if (currentChunk){
        chunked.push(currentChunk);
    }

    return chunked;
}

const chunkedConversations = computed(() => {
    return chunkBy(messages.value, (prev, curr) => prev.sender_username === curr.sender_username)
        .map(chunk => {
            const senderOmitted = chunk.map(message => {
                return {
                    text: message.text,
                    sent_at: new Date(Date.parse(message.sent_at))
                };
            });

            let timeChunked = chunkBy(senderOmitted, (prev, curr) => {
                return prev.sent_at.getHours() === curr.sent_at.getHours() && prev.sent_at.getMinutes() === curr.sent_at.getMinutes();
            });

            return {
                senderUsername: chunk[0].sender_username,
                timeChunks: timeChunked
            };
        });
});

function findMemberByUsername(username){
    return conversation.value.members.find(member => member.username === username);
}

function toHourMinuteFormat(date){
    const padZero = (str) => str.toString().padStart(2, '0');
    return `${padZero(date.getHours())}:${padZero(date.getMinutes())}`;
}

function sendMessage(){
    const trimmedMessage = currentMessage.value.trim();
    if (trimmedMessage === ''){
        return;
    }

    fetch(`https://localhost:8443/api/conversation/${props.conversation_id}/message`, { 
        method: 'POST',
        mode: 'cors',
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json'
        },
        body: trimmedMessage
    })
    .then(response => response.json())
    .then(data => {
        messages.value.push(data);

        socket.send(JSON.stringify({
            message: data
        }));
        currentMessage.value = '';

        // Should not directly call scrollMessageSectionToEnd() here, because the DOM is not updated yet.
        nextTick(() => {
            scrollMessageSectionToEnd();
        });
    });
}

function scrollMessageSectionToEnd(){
    const nestedElement = document.getElementById('message_section');
    nestedElement.scrollTo(0, nestedElement.scrollHeight);
}
</script>

<template>
    <div v-if="isDataLoaded" class="w-full flex flex-col gap-y-2 overflow-y-auto">
        <div class="flex gap-x-4">
            <ConversationThumbnail class="w-12 h-12" :members="conversation.members" :selfUsername="self.username"/>

            <div class="flex flex-col">
                <h2 class="text-gray-100 font-bold">{{ conversation.name }}</h2>
                <div class="flex items-center gap-x-1">
                    <img class="w-4 h-4" src="/src/assets/user.png" alt="User icon">
                    <p class="text-gray-300 text-sm">{{ conversation.members.length }}</p>
                </div>
            </div>
        </div>

        <hr>

        <div id="message_section" class="grow flex flex-col gap-y-2 overflow-y-auto">
            <div v-for="senderChunk in chunkedConversations">
                <!-- Message from self is on the right side, and no profile picture shown. -->
                <div v-if="senderChunk.senderUsername === props.self.username" class="flex flex-col gap-y-1 items-end">
                    <div class="flex flex-col items-end gap-y-1 max-w-[60%]" v-for="timeChunk in senderChunk.timeChunks">
                        <div class="flex items-end gap-x-1" v-for="message, idx in timeChunk" :key="message.id">
                            <p v-if="idx === timeChunk.length - 1" class="text-gray-400 text-xs">{{ toHourMinuteFormat(message.sent_at) }}</p>
                            <div class="text-white bg-gray-700 rounded-md px-2 py-1">
                                {{ message.text }}
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Message from other members are on the left side, and profile picture shown. -->
                <div v-else class="flex gap-x-2">
                    <ProfileThumbnail class="w-8 h-8" :user="findMemberByUsername(senderChunk.senderUsername)"/>
                    <div class="flex flex-col gap-y-1 max-w-[60%]">
                        <span class="text-white">{{ findMemberByUsername(senderChunk.senderUsername)?.nickname }}</span>
                        <div class="flex flex-col gap-y-1" v-for="timeChunk in senderChunk.timeChunks">
                            <div class="flex items-end gap-x-1" v-for="message, idx in timeChunk" :key="message.id">
                                <div class="text-white bg-gray-700 rounded-md px-2 py-1" >
                                    {{ message.text }}
                                </div>
                                <p v-if="idx === timeChunk.length - 1" 
                                    class="text-gray-400 text-xs">{{ toHourMinuteFormat(message.sent_at) }}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <form class="flex gap-x-2" @submit.prevent="sendMessage">
            <input class="flex-grow rounded-lg p-2 bg-gray-800 text-gray-100" 
                v-model="currentMessage"
                type="text" 
                name="message" 
                id="message" 
                placeholder="Type a message...">
            <button type="submit" class="bg-blue-600 text-gray-200 px-4 py-2 rounded-lg">Send</button>
        </form>
    </div>
</template>