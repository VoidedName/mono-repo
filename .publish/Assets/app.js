const list = document.getElementById('projectList');
const items = document.querySelectorAll('.project-item');
const help = document.getElementById('helpOverlay');
const helpToggle = document.getElementById('helpToggle');
let activeIndex = 0;

function updateActive() {
    items.forEach((item, index) => {
        const isActive = index === activeIndex;
        if (isActive) {
            item.classList.add('active');
            item.setAttribute('aria-selected', 'true');
            list.setAttribute('aria-activedescendant', item.id);
            item.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        } else {
            item.classList.remove('active');
            item.setAttribute('aria-selected', 'false');
        }
    });
}

function toggleHelp() {
    const isVisible = help.style.display === 'flex';
    help.style.display = isVisible ? 'none' : 'flex';
}

// Mouse interaction
items.forEach((item, index) => {
    item.addEventListener('mouseenter', () => {
        activeIndex = index;
        updateActive();
    });
    item.addEventListener('click', () => {
        window.location.href = item.dataset.url;
    });
});

helpToggle.addEventListener('click', (e) => {
    e.stopPropagation();
    toggleHelp();
});

help.addEventListener('click', () => {
    help.style.display = 'none';
});

document.querySelector('.help-content').addEventListener('click', (e) => {
    e.stopPropagation();
});

// Keyboard interaction
window.addEventListener('keydown', (e) => {
    if (help.style.display === 'flex') {
        help.style.display = 'none';
        return;
    }

    switch(e.key.toLowerCase()) {
        case 'arrowup':
        case 'k':
            e.preventDefault();
            activeIndex = Math.max(0, activeIndex - 1);
            updateActive();
            break;
        case 'arrowdown':
        case 'j':
            e.preventDefault();
            activeIndex = Math.min(items.length - 1, activeIndex + 1);
            updateActive();
            break;
        case 'enter':
            window.location.href = items[activeIndex].dataset.url;
            break;
        case 'h':
        case '?':
            help.style.display = 'flex';
            break;
    }
});

// Initial state
if (items.length > 0) updateActive();
