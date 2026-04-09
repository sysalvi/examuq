<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Model;

class SessionEvent extends Model
{
    use HasFactory;

    protected $fillable = [
        'session_id',
        'event_type',
        'severity',
        'payload_json',
        'event_at',
    ];

    protected $casts = [
        'payload_json' => 'array',
        'event_at' => 'datetime',
    ];

    public function session(): BelongsTo
    {
        return $this->belongsTo(ExamSession::class, 'session_id');
    }
}
