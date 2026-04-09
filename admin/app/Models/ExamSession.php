<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Model;

class ExamSession extends Model
{
    use HasFactory;

    protected $fillable = [
        'exam_id',
        'display_name',
        'class_room',
        'client_type',
        'device_id',
        'status',
        'started_at',
        'last_heartbeat_at',
        'ended_at',
        'ip_address',
        'user_agent',
    ];

    protected $casts = [
        'started_at' => 'datetime',
        'last_heartbeat_at' => 'datetime',
        'ended_at' => 'datetime',
    ];

    public function exam(): BelongsTo
    {
        return $this->belongsTo(Exam::class);
    }

    public function events(): HasMany
    {
        return $this->hasMany(SessionEvent::class, 'session_id');
    }

    public function launchTokens(): HasMany
    {
        return $this->hasMany(LaunchToken::class);
    }
}
